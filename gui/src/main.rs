// Cloaker's desktop GUI. Drop a file on the window (or use File > Open) and it is encrypted or
// decrypted depending on what the file turns out to be. All of the real work lives in the
// `cloaker` core crate; this is only the window around it.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use cloaker::{check_password_length, detect_mode, main_routine, Config, Mode, Ui};
use eframe::egui;

const WINDOW_SIZE: [f32; 2] = [440.0, 420.0];
const DROP_PROMPT: &str = "Drop a normal file here to encrypt\n\nor an encrypted file to decrypt";

fn main() -> eframe::Result {
    let initial_file = match initial_file(std::env::args_os().skip(1)) {
        Ok(file) => file,
        Err(usage) => {
            // a Windows GUI binary has no console attached, so printing can fail: ignore it
            // rather than panicking on the way out
            let _ = writeln!(std::io::stdout(), "{usage}");
            return Ok(());
        }
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(WINDOW_SIZE)
            .with_min_inner_size([320.0, 300.0])
            .with_drag_and_drop(true)
            .with_icon(icon()),
        ..Default::default()
    };
    eframe::run_native(
        "Cloaker",
        options,
        Box::new(move |_cc| Ok(Box::new(CloakerApp::new(initial_file.clone())))),
    )
}

// `cloaker somefile.txt` opens that file straight away, so the app can be used as the handler for
// a file type or from a file manager's "Open with". Returns Err with the usage text for -h/--help.
fn initial_file(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<Option<PathBuf>, String> {
    let Some(arg) = args.next() else {
        return Ok(None);
    };
    let lossy = arg.to_string_lossy().to_string();
    match lossy.as_str() {
        "-h" | "--help" => Err(format!(
            "Cloaker {}\n\nusage: cloaker [FILE]\n\nOpens the window, encrypting or decrypting \
             FILE if one is given. Files can also be dropped on the window or opened from the \
             File menu.",
            env!("CARGO_PKG_VERSION")
        )),
        "-V" | "--version" => Err(format!("Cloaker {}", env!("CARGO_PKG_VERSION"))),
        _ if lossy.starts_with('-') => Err(format!(
            "cloaker: unrecognized option `{lossy}`\nusage: cloaker [FILE]"
        )),
        _ => Ok(Some(PathBuf::from(arg))),
    }
}

fn icon() -> egui::IconData {
    eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
        .unwrap_or_else(|_| egui::IconData::default())
}

// messages from the worker thread
enum Msg {
    Progress(i32),
    Done(Result<String, String>),
}

// progress reporting from core, bridged onto the UI thread
struct ProgressUpdater {
    tx: Sender<Msg>,
    ctx: egui::Context,
}

impl Ui for ProgressUpdater {
    fn output(&self, percentage: i32) {
        let _ = self.tx.send(Msg::Progress(percentage));
        self.ctx.request_repaint();
    }
}

enum Stage {
    Idle,
    Password {
        path: PathBuf,
        mode: Mode,
        password: String,
        confirm: String,
        error: Option<String>,
        // the password box is focused when the prompt first appears, and never again: asking for
        // focus every frame would yank it back out of the confirm box as soon as it was clicked
        focus_given: bool,
    },
    Working {
        rx: Receiver<Msg>,
    },
    Done {
        message: String,
        ok: bool,
    },
}

// what the bottom progress bar is showing: kept outside `Stage` so that it stays on screen with
// its final value while the result dialog is up, the way the Qt version behaved
struct Progress {
    percent: i32,
    mode: Mode,
}

struct CloakerApp {
    stage: Stage,
    notice: Option<String>,
    show_about: bool,
    progress: Option<Progress>,
}

impl Default for CloakerApp {
    fn default() -> Self {
        CloakerApp {
            stage: Stage::Idle,
            notice: None,
            show_about: false,
            progress: None,
        }
    }
}

impl CloakerApp {
    fn new(initial_file: Option<PathBuf>) -> Self {
        let mut app = CloakerApp::default();
        if let Some(path) = initial_file {
            app.open(path);
        }
        app
    }
}

impl eframe::App for CloakerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }
}

impl CloakerApp {
    fn draw(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.poll_worker(&ctx);
        self.handle_drops(&ctx);

        egui::Panel::top("menu_bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui
                        .add_enabled(self.is_idle(), egui::Button::new("Open…"))
                        .clicked()
                    {
                        ui.close();
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("Select a file to encrypt or decrypt")
                            .pick_file()
                        {
                            self.open(path);
                        }
                    }
                    if ui.button("Quit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About Cloaker").clicked() {
                        ui.close();
                        self.show_about = true;
                    }
                });
            });
        });

        // the progress bar sits at the bottom of the window, under the drop area, and appears
        // only once there is something to report
        if let Some(progress) = &self.progress {
            let verb = match progress.mode {
                Mode::Encrypt => "Encrypting",
                Mode::Decrypt => "Decrypting",
            };
            egui::Panel::bottom("progress").show(ui, |ui| {
                ui.add_space(4.0);
                ui.add(
                    egui::ProgressBar::new(progress.percent as f32 / 100.0)
                        .text(format!("{verb}: {}%", progress.percent)),
                );
                ui.add_space(4.0);
            });
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(8.0);
            let text = if matches!(self.stage, Stage::Working { .. }) {
                "Working…"
            } else {
                DROP_PROMPT
            };
            drop_area(ui, text);
        });

        // dialogs sit on top of the window, as the Qt version's message boxes did
        self.password_modal(&ctx);
        self.result_modal(&ctx);
        self.notice_modal(&ctx);
        self.about_modal(&ctx);
    }

    fn is_idle(&self) -> bool {
        matches!(self.stage, Stage::Idle)
    }

    // a file arrived, by drop, dialog or command line: work out what to do with it and ask for a
    // password
    fn open(&mut self, path: PathBuf) {
        self.notice = None;
        if !path.is_file() {
            self.notice = Some(format!("{} is not a file.", display(&path)));
            return;
        }
        match detect_mode(&path) {
            Ok(mode) => {
                self.stage = Stage::Password {
                    path,
                    mode,
                    password: String::new(),
                    confirm: String::new(),
                    error: None,
                    focus_given: false,
                }
            }
            Err(e) => self.notice = Some(format!("Could not read {}: {}", display(&path), e)),
        }
    }

    fn handle_drops(&mut self, ctx: &egui::Context) {
        if !self.is_idle() {
            return;
        }
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if dropped.is_empty() {
            return;
        }
        if dropped.len() > 1 {
            self.notice =
                Some("Only one file at a time can be encrypted or decrypted.".to_string());
            return;
        }
        let path = dropped[0].path().to_path_buf();
        if path.is_file() {
            self.open(path);
        } else {
            self.notice = Some(
                "Only single files can be processed. To encrypt a folder, please wrap it in a \
                 .zip file or similar archive/compression format first."
                    .to_string(),
            );
        }
    }

    fn poll_worker(&mut self, ctx: &egui::Context) {
        let mut finished = None;
        if let Stage::Working { rx } = &mut self.stage {
            for msg in rx.try_iter() {
                match msg {
                    Msg::Progress(p) => {
                        if let Some(progress) = &mut self.progress {
                            progress.percent = p;
                        }
                    }
                    Msg::Done(result) => finished = Some(result),
                }
            }
        }
        if let Some(result) = finished {
            self.stage = match result {
                Ok(message) => Stage::Done { message, ok: true },
                Err(message) => Stage::Done { message, ok: false },
            };
            ctx.request_repaint();
        }
    }

    fn password_modal(&mut self, ctx: &egui::Context) {
        if !matches!(self.stage, Stage::Password { .. }) {
            return;
        }
        let closed = egui::Modal::new(egui::Id::new("password_modal"))
            .show(ctx, |ui| {
                ui.set_max_width(340.0);
                self.password_panel(ui);
            })
            .should_close();
        if closed {
            // escape or a click outside means cancel, as it did with the Qt dialogs
            self.stage = Stage::Idle;
        }
    }

    fn password_panel(&mut self, ui: &mut egui::Ui) {
        let Stage::Password {
            path,
            mode,
            password,
            confirm,
            error,
            focus_given,
        } = &mut self.stage
        else {
            return;
        };
        let encrypting = matches!(mode, Mode::Encrypt);
        let mut submitted = false;
        let mut cancelled = false;
        let mut focus_confirm = false;
        let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

        ui.vertical_centered(|ui| {
            ui.heading(if encrypting {
                "Encrypt file"
            } else {
                "Decrypt file"
            });
            ui.add_space(4.0);
            ui.label(display(path));
            ui.add_space(12.0);
        });

        egui::Grid::new("password_grid")
            .num_columns(2)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                let label = ui.label("Password:");
                let field = ui
                    .add(
                        egui::TextEdit::singleline(password)
                            .password(true)
                            .desired_width(220.0),
                    )
                    .labelled_by(label.id);
                if !*focus_given {
                    field.request_focus();
                    *focus_given = true;
                }
                if field.lost_focus() && enter_pressed {
                    // enter moves on to the confirmation when encrypting, and submits when there
                    // isn't one to fill in
                    if encrypting {
                        focus_confirm = true;
                    } else {
                        submitted = true;
                    }
                }
                ui.end_row();

                if encrypting {
                    let label = ui.label("Confirm:");
                    let confirm_field = ui
                        .add(
                            egui::TextEdit::singleline(confirm)
                                .password(true)
                                .desired_width(220.0),
                        )
                        .labelled_by(label.id);
                    if focus_confirm {
                        confirm_field.request_focus();
                    }
                    if confirm_field.lost_focus() && enter_pressed {
                        submitted = true;
                    }
                    ui.end_row();
                }
            });

        if encrypting {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(
                    "Minimum 12 characters. A passphrase of several words is \
                                     stronger than a short password.",
                )
                .small()
                .weak(),
            );
        }

        if let Some(error) = error {
            ui.add_space(6.0);
            ui.colored_label(ui.visuals().error_fg_color, error.clone());
        }

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui
                .button(if encrypting { "Encrypt" } else { "Decrypt" })
                .clicked()
            {
                submitted = true;
            }
            if ui.button("Cancel").clicked() {
                cancelled = true;
            }
        });

        if cancelled {
            self.stage = Stage::Idle;
            return;
        }
        if submitted {
            let ctx = ui.ctx().clone();
            self.submit_password(&ctx);
        }
    }

    // validate what was typed, ask where to save, then hand off to a worker thread
    fn result_modal(&mut self, ctx: &egui::Context) {
        let Stage::Done { message, ok } = &self.stage else {
            return;
        };
        let (message, ok) = (message.clone(), *ok);
        let response = egui::Modal::new(egui::Id::new("result_modal")).show(ctx, |ui| {
            ui.set_max_width(340.0);
            if ok {
                ui.heading("Success");
            } else {
                ui.heading(egui::RichText::new("Failed").color(ui.visuals().error_fg_color));
            }
            ui.add_space(8.0);
            ui.label(message);
            ui.add_space(12.0);
            ui.horizontal(|ui| ui.button("OK").clicked()).inner
        });
        if response.inner || response.should_close() {
            self.stage = Stage::Idle;
            self.progress = None;
        }
    }

    fn notice_modal(&mut self, ctx: &egui::Context) {
        let Some(notice) = self.notice.clone() else {
            return;
        };
        let response = egui::Modal::new(egui::Id::new("notice_modal")).show(ctx, |ui| {
            ui.set_max_width(340.0);
            ui.label(notice);
            ui.add_space(12.0);
            ui.horizontal(|ui| ui.button("OK").clicked()).inner
        });
        if response.inner || response.should_close() {
            self.notice = None;
        }
    }

    fn submit_password(&mut self, ctx: &egui::Context) {
        let Stage::Password {
            path,
            mode,
            password,
            confirm,
            error,
            ..
        } = &mut self.stage
        else {
            return;
        };

        if matches!(mode, Mode::Encrypt) {
            if let Err(e) = check_password_length(password) {
                *error = Some(e);
                return;
            }
            if password != confirm {
                *error = Some("Passwords do not match.".to_string());
                return;
            }
        } else if password.is_empty() {
            *error = Some("Please enter the password this file was encrypted with.".to_string());
            return;
        }

        let (path, mode, password) = (path.clone(), mode.clone(), password.clone());
        let Some(out_path) = save_dialog(&path, &mode) else {
            return; // user cancelled the save dialog; leave the password on screen
        };
        self.spawn(ctx, path, out_path, mode, password);
    }

    fn spawn(
        &mut self,
        ctx: &egui::Context,
        in_path: PathBuf,
        out_path: PathBuf,
        mode: Mode,
        password: String,
    ) {
        let rx = run_in_background(ctx, in_path, out_path, mode.clone(), password);
        self.progress = Some(Progress { percent: 0, mode });
        self.stage = Stage::Working { rx };
    }

    fn about_modal(&mut self, ctx: &egui::Context) {
        if !self.show_about {
            return;
        }
        let response = egui::Modal::new(egui::Id::new("about_modal")).show(ctx, |ui| {
            ui.set_max_width(360.0);
            ui.heading(concat!("Cloaker ", env!("CARGO_PKG_VERSION")));
            ui.add_space(4.0);
            ui.label("Copyright © 2026 Theron Spiegl");
            ui.label("Licensed under the GNU General Public License v3.0");
            ui.hyperlink("https://cloaker.spiegl.dev");
            ui.hyperlink_to("theron@spiegl.dev", "mailto:theron@spiegl.dev");
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(
                    "WARNING: if you encrypt a file and lose or forget the password, the file \
                     cannot be recovered.",
                )
                .strong(),
            );
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Backward compatibility notes").strong());
            ui.label(
                "To decrypt a file made with version 1.0 or 1.1 of Cloaker (with Encrypt and \
                 Decrypt buttons), the filename must end with the \".cloaker\" extension. \
                 Files encrypted with later versions are not subject to this restriction.",
            );
            ui.label(
                "Cloaker 5 writes the same file format as Cloaker 4, so the two can read each \
                 other's files. Both can also decrypt files from earlier versions, but versions \
                 before 4 cannot read files written by 4 or 5.",
            );
            ui.add_space(12.0);
            ui.horizontal(|ui| ui.button("Close").clicked()).inner
        });
        if response.inner || response.should_close() {
            self.show_about = false;
        }
    }
}

// runs the encryption or decryption on a worker thread so the window stays responsive
fn run_in_background(
    ctx: &egui::Context,
    in_path: PathBuf,
    out_path: PathBuf,
    mode: Mode,
    password: String,
) -> Receiver<Msg> {
    let (tx, rx) = channel();
    let ui = Box::new(ProgressUpdater {
        tx: tx.clone(),
        ctx: ctx.clone(),
    });
    let config = Config::new(
        &mode,
        password,
        Some(display(&in_path)),
        Some(display(&out_path)),
        ui,
    );
    let out_name = display(&out_path);
    let ctx = ctx.clone();
    thread::spawn(move || {
        let verb = match config.mode {
            Mode::Encrypt => "encrypted",
            Mode::Decrypt => "decrypted",
        };
        let result = match main_routine(&config) {
            Ok(()) => Ok(format!("{} has been {}.", out_name, verb)),
            Err(e) => Err(e.to_string()),
        };
        let _ = tx.send(Msg::Done(result));
        ctx.request_repaint();
    });
    rx
}

fn drop_area(ui: &mut egui::Ui, text: &str) {
    let rect = ui.available_rect_before_wrap();
    ui.painter().rect_filled(
        rect,
        egui::CornerRadius::same(6),
        ui.visuals().extreme_bg_color,
    );
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::same(6),
        ui.visuals().widgets.noninteractive.bg_stroke,
        egui::StrokeKind::Inside,
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(rect.height() / 3.0);
            ui.label(egui::RichText::new(text).size(15.0));
        });
    });
}

// default output name comes from core, so the GUI and the CLI agree
fn default_output_path(input: &Path, mode: &Mode) -> PathBuf {
    let parent = input
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    cloaker::generate_default_filename(mode, parent.clone(), Some(&display(input)))
        .unwrap_or_else(|_| parent.join("cloaker-output"))
}

fn save_dialog(input: &Path, mode: &Mode) -> Option<PathBuf> {
    let parent = input
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let default = default_output_path(input, mode);
    let title = match mode {
        Mode::Encrypt => "Save encrypted file",
        Mode::Decrypt => "Save decrypted file",
    };
    rfd::FileDialog::new()
        .set_title(title)
        .set_directory(&parent)
        .set_file_name(
            default
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default(),
        )
        .save_file()
}

fn display(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::kittest::Queryable;
    use egui_kittest::Harness;
    use std::fs::{create_dir_all, read, remove_dir_all, write};
    use std::sync::mpsc::RecvTimeoutError;
    use std::time::Duration;

    const PASSWORD: &str = "a good long password";

    fn harness<'a>() -> Harness<'a, CloakerApp> {
        Harness::new_ui_state(
            |ui, app: &mut CloakerApp| app.draw(ui),
            CloakerApp::default(),
        )
    }

    // a scratch directory that cleans up after itself
    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(name: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("cloaker-gui-{}-{}", name, std::process::id()));
            let _ = remove_dir_all(&path);
            create_dir_all(&path).unwrap();
            TempDir { path }
        }

        fn file(&self, name: &str) -> PathBuf {
            self.path.join(name)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = remove_dir_all(&self.path);
        }
    }

    fn password_stage(mode: Mode, password: &str, confirm: &str) -> Stage {
        Stage::Password {
            path: PathBuf::from("/tmp/example.txt"),
            mode,
            password: password.to_string(),
            confirm: confirm.to_string(),
            error: None,
            focus_given: false,
        }
    }

    fn args(list: &[&str]) -> impl Iterator<Item = std::ffi::OsString> {
        list.iter()
            .map(std::ffi::OsString::from)
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn a_file_argument_is_opened_at_startup() {
        let dir = TempDir::new("argument");
        let file = dir.file("in.txt");
        write(&file, b"plain text, so this should be an encrypt").unwrap();

        assert_eq!(
            initial_file(args(&[file.to_str().unwrap()])).unwrap(),
            Some(file.clone())
        );
        let app = CloakerApp::new(Some(file.clone()));
        match &app.stage {
            Stage::Password { path, mode, .. } => {
                assert_eq!(path, &file);
                assert!(matches!(mode, Mode::Encrypt));
            }
            _ => panic!("expected the password prompt for the file given on the command line"),
        }
        assert!(app.notice.is_none());
    }

    #[test]
    fn a_missing_file_argument_is_reported_rather_than_crashing() {
        let app = CloakerApp::new(Some(PathBuf::from("/definitely/not/here.txt")));
        assert!(matches!(app.stage, Stage::Idle));
        assert!(app.notice.unwrap().contains("is not a file"));
    }

    #[test]
    fn help_and_version_flags_print_instead_of_opening_a_window() {
        assert!(initial_file(args(&["--help"]))
            .unwrap_err()
            .contains("usage: cloaker [FILE]"));
        assert!(initial_file(args(&["-V"]))
            .unwrap_err()
            .contains("Cloaker 5.0"));
        assert!(initial_file(args(&["--nonsense"]))
            .unwrap_err()
            .contains("unrecognized option"));
        assert_eq!(initial_file(args(&[])).unwrap(), None);
    }

    #[test]
    fn idle_shows_the_drop_prompt() {
        let mut h = harness();
        h.run();
        assert!(h
            .query_by_label_contains("Drop a normal file here")
            .is_some());
    }

    // these two click the submit button on purpose: validation fails first, so no save dialog opens
    #[test]
    fn typing_in_the_confirm_box_stays_in_the_confirm_box() {
        let mut h = harness();
        h.state_mut().stage = password_stage(Mode::Encrypt, "", "");
        h.run();

        h.get_by_label("Confirm:").click();
        h.run();
        h.get_by_label("Confirm:").type_text("second attempt");
        h.run();

        match &h.state().stage {
            Stage::Password {
                password, confirm, ..
            } => {
                assert_eq!(confirm, "second attempt");
                assert!(
                    password.is_empty(),
                    "typing in the confirm box went to the password box: {password:?}"
                );
            }
            _ => panic!("expected to still be on the password prompt"),
        }
    }

    #[test]
    fn the_password_box_is_focused_when_the_prompt_appears() {
        let mut h = harness();
        h.state_mut().stage = password_stage(Mode::Encrypt, "", "");
        h.run();
        h.get_by_label("Password:")
            .type_text("typed without clicking");
        h.run();

        match &h.state().stage {
            Stage::Password { password, .. } => {
                assert_eq!(password, "typed without clicking")
            }
            _ => panic!("expected to still be on the password prompt"),
        }
    }

    #[test]
    fn the_progress_bar_shows_while_working_and_goes_away_with_the_result() {
        let mut h = harness();
        let (_tx, rx) = channel();
        h.state_mut().stage = Stage::Working { rx };
        h.state_mut().progress = Some(Progress {
            percent: 42,
            mode: Mode::Encrypt,
        });
        h.run();
        assert!(
            h.query_by_label_contains("Encrypting: 42%").is_some(),
            "no progress bar while working"
        );

        // it belongs at the bottom of the window, under the drop area, as in the Qt version
        let bar = h.get_by_label_contains("Encrypting: 42%").rect();
        let drop_area = h.get_by_label_contains("Working").rect();
        let window = h.ctx.content_rect();
        assert!(
            bar.top() >= drop_area.bottom(),
            "the progress bar ({bar:?}) should sit below the drop area ({drop_area:?})"
        );
        assert!(
            window.bottom() - bar.bottom() < 24.0,
            "the progress bar ({bar:?}) should be at the bottom of the window ({window:?})"
        );

        // the bar keeps its final value while the result dialog is up, then both clear together
        h.state_mut().stage = Stage::Done {
            message: "all done".to_string(),
            ok: true,
        };
        h.run();
        assert!(h.query_by_label_contains("Encrypting: 42%").is_some());
        h.get_by_role_and_label(egui::accesskit::Role::Button, "OK")
            .click();
        h.run();
        assert!(h.query_by_label_contains("Encrypting").is_none());
        assert!(h.state().progress.is_none());
        assert!(matches!(h.state().stage, Stage::Idle));
    }

    #[test]
    fn cancelling_the_password_prompt_returns_to_the_drop_area() {
        let mut h = harness();
        h.state_mut().stage = password_stage(Mode::Encrypt, "", "");
        h.run();
        h.get_by_role_and_label(egui::accesskit::Role::Button, "Cancel")
            .click();
        h.run();
        assert!(matches!(h.state().stage, Stage::Idle));
        assert!(h
            .query_by_label_contains("Drop a normal file here")
            .is_some());
    }

    #[test]
    fn short_password_is_rejected_before_any_work_starts() {
        let mut h = harness();
        h.state_mut().stage = password_stage(Mode::Encrypt, "short", "short");
        h.run();
        h.get_by_role_and_label(egui::accesskit::Role::Button, "Encrypt")
            .click();
        h.run();
        assert!(h
            .query_by_label_contains("at least 12 characters")
            .is_some());
        assert!(matches!(h.state().stage, Stage::Password { .. }));
    }

    #[test]
    fn mismatched_confirmation_is_rejected() {
        let mut h = harness();
        h.state_mut().stage = password_stage(Mode::Encrypt, PASSWORD, "something else entirely");
        h.run();
        h.get_by_role_and_label(egui::accesskit::Role::Button, "Encrypt")
            .click();
        h.run();
        assert!(h.query_by_label_contains("do not match").is_some());
        assert!(matches!(h.state().stage, Stage::Password { .. }));
    }

    #[test]
    fn file_menu_holds_the_file_actions() {
        let mut h = harness();
        h.run();
        h.get_by_label("File").click();
        h.run();
        assert!(
            h.query_by_label_contains("Open").is_some(),
            "File menu has no Open item"
        );
        assert!(
            h.query_by_label("Quit").is_some(),
            "File menu has no Quit item"
        );
        assert!(
            h.query_by_label("About Cloaker").is_none(),
            "the File menu is showing the Help menu's contents"
        );
    }

    #[test]
    fn about_window_opens_from_the_help_menu() {
        let mut h = harness();
        h.run();
        h.get_by_label("Help").click();
        h.run();
        h.get_by_label("About Cloaker").click();
        h.run();
        assert!(h
            .query_by_label_contains("Copyright © 2026 Theron Spiegl")
            .is_some());
        assert!(h.query_by_label_contains("theron@spiegl.dev").is_some());
    }

    #[test]
    fn default_output_names_match_the_cli() {
        assert_eq!(
            default_output_path(Path::new("/tmp/notes.txt"), &Mode::Encrypt),
            PathBuf::from("/tmp/notes.txt.cloaker")
        );
        assert_eq!(
            default_output_path(Path::new("/tmp/notes.txt.cloaker"), &Mode::Decrypt),
            PathBuf::from("/tmp/notes.txt")
        );
    }

    #[test]
    fn background_work_reports_progress_and_round_trips() {
        let dir = TempDir::new("worker");
        let plaintext: Vec<u8> = (0..300_000u32).map(|i| (i % 251) as u8).collect();
        let original = dir.file("in.bin");
        write(&original, &plaintext).unwrap();
        let encrypted = dir.file("in.cloaker");
        let ctx = egui::Context::default();

        let rx = run_in_background(
            &ctx,
            original.clone(),
            encrypted.clone(),
            Mode::Encrypt,
            PASSWORD.to_string(),
        );
        let (progress, done) = collect(rx);
        assert!(
            progress.contains(&100),
            "progress never reached 100: {progress:?}"
        );
        assert!(done.unwrap().contains("has been encrypted"));

        // and the GUI's own output decrypts back to the original bytes
        let decrypted = dir.file("out.bin");
        let rx = run_in_background(
            &ctx,
            encrypted,
            decrypted.clone(),
            Mode::Decrypt,
            PASSWORD.to_string(),
        );
        let (_, done) = collect(rx);
        assert!(done.unwrap().contains("has been decrypted"));
        assert_eq!(read(&decrypted).unwrap(), plaintext);
    }

    #[test]
    fn wrong_password_reports_failure_and_leaves_no_file() {
        let dir = TempDir::new("wrong-password");
        let original = dir.file("in.bin");
        write(&original, b"some secret bytes").unwrap();
        let encrypted = dir.file("in.cloaker");
        let ctx = egui::Context::default();

        let rx = run_in_background(
            &ctx,
            original,
            encrypted.clone(),
            Mode::Encrypt,
            PASSWORD.to_string(),
        );
        collect(rx).1.unwrap();

        let decrypted = dir.file("out.bin");
        let rx = run_in_background(
            &ctx,
            encrypted,
            decrypted.clone(),
            Mode::Decrypt,
            "not the password".to_string(),
        );
        let err = collect(rx).1.unwrap_err();
        assert!(
            err.contains("Incorrect password"),
            "unexpected error: {err}"
        );
        assert!(!decrypted.exists());
    }

    // drains the worker channel until it reports that it finished
    fn collect(rx: Receiver<Msg>) -> (Vec<i32>, Result<String, String>) {
        let mut progress = Vec::new();
        loop {
            match rx.recv_timeout(Duration::from_secs(60)) {
                Ok(Msg::Progress(p)) => progress.push(p),
                Ok(Msg::Done(result)) => return (progress, result),
                Err(RecvTimeoutError::Timeout) => panic!("worker timed out"),
                Err(RecvTimeoutError::Disconnected) => panic!("worker disappeared"),
            }
        }
    }
}
