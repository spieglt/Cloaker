use std::error::Error;
use std::fs::{canonicalize, remove_file, File};
use std::io::prelude::*;
use std::path::Path;

#[derive(Clone, Debug)]
pub enum Mode {
    Encrypt,
    Decrypt,
}

pub struct Config {
    pub mode: Mode,
    pub password: String,
    pub filename: Option<String>,
    pub out_file: Option<String>,
    pub ui: Box<dyn Ui + Send>,
}

pub trait Ui {
    fn output(&self, percentage: i32);
}

impl Config {
    pub fn new(
        mode: &Mode,
        password: String,
        filename: Option<String>,
        out_file: Option<String>,
        ui: Box<dyn Ui + Send>,
    ) -> Self {
        Config {
            mode: mode.clone(),
            password,
            filename,
            out_file,
            ui,
        }
    }
}

/// Decides whether a file should be encrypted or decrypted: files named `.cloaker` are always
/// treated as encrypted (cloaker 1.0 and 1.1 wrote no signature), otherwise the first four bytes
/// are checked against the current and legacy signatures. A file too short to hold a signature is
/// something to encrypt; anything else that can't be read is reported to the caller.
pub fn detect_mode(path: &Path) -> std::io::Result<Mode> {
    if path
        .to_string_lossy()
        .to_lowercase()
        .ends_with(crate::FILE_EXTENSION)
    {
        return Ok(Mode::Decrypt);
    }
    let mut file = File::open(path)?;
    let mut first_four = [0u8; 4];
    match file.read_exact(&mut first_four) {
        Ok(()) => (),
        Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(Mode::Encrypt),
        Err(e) => return Err(e),
    }
    if first_four == crate::SIGNATURE || first_four == crate::legacy::SIGNATURE {
        Ok(Mode::Decrypt)
    } else {
        Ok(Mode::Encrypt)
    }
}

// what to do with the input, decided before the output file is created
enum Work {
    Encrypt,
    Decrypt,
    // Some(first_four) if the file had no signature, meaning those bytes are the start of the salt
    Legacy(Option<[u8; 4]>),
}

pub fn main_routine(c: &Config) -> Result<(), Box<dyn Error>> {
    // creating the output file truncates it, so make sure it isn't also the input
    if let (Some(in_name), Some(out_name)) = (&c.filename, &c.out_file) {
        if same_file(in_name, out_name) {
            return Err(
                "Input and output are the same file. Please choose another output path.".into(),
            );
        }
    }

    let in_file = match &c.filename {
        Some(s) => Some(File::open(s)?),
        None => None,
    };
    let filesize = match &in_file {
        Some(f) => Some(f.metadata()?.len() as usize),
        None => None,
    };
    let mut input = file_or_stdin(in_file);

    // start reading stream before creating the output file, so that an input that isn't
    // long enough to be an encrypted file can't truncate an existing file.
    // legacy decrypt might need a first-four-bytes param.
    let work = match c.mode {
        Mode::Encrypt => Work::Encrypt,
        Mode::Decrypt => {
            let mut first_four = [0u8; 4];
            input.read_exact(&mut first_four)?;
            match first_four {
                crate::SIGNATURE => Work::Decrypt,
                crate::legacy::SIGNATURE => Work::Legacy(None),
                _ => Work::Legacy(Some(first_four)),
            }
        }
    };

    let out_file = match &c.out_file {
        Some(s) => Some(File::create(s)?),
        None => None,
    };
    let mut output = file_or_stdout(out_file);

    let result = match work {
        Work::Encrypt => crate::encrypt(
            &mut input,
            &mut output,
            &c.password,
            c.ui.as_ref(),
            filesize,
        ),
        Work::Decrypt => crate::decrypt(
            &mut input,
            &mut output,
            &c.password,
            c.ui.as_ref(),
            filesize,
        ),
        Work::Legacy(first_four) => crate::legacy::decrypt(
            &mut input,
            &mut output,
            &c.password,
            c.ui.as_ref(),
            filesize,
            first_four,
        ),
    };

    if let Err(e) = result {
        // close the file before removing it, otherwise the removal fails on Windows
        drop(output);
        if let Some(out_file) = &c.out_file {
            remove_file(out_file)
                .map_err(|e2| format!("{}. Could not delete output file: {}.", e, e2))?;
        }
        return Err(e);
    }
    Ok(())
}

// true only if both paths exist and resolve to the same file
fn same_file(a: &str, b: &str) -> bool {
    match (canonicalize(a), canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

fn file_or_stdin(reader: Option<File>) -> Box<dyn Read> {
    match reader {
        Some(file) => Box::new(file),
        None => Box::new(std::io::stdin()),
    }
}

fn file_or_stdout(writer: Option<File>) -> Box<dyn Write> {
    match writer {
        Some(file) => Box::new(file),
        None => Box::new(std::io::stdout()),
    }
}
