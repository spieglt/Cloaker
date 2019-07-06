use cloaker::*;

use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::exit;
use rpassword;
use sodiumoxide;

const FILE_EXTENSION: &str = ".cloaker";

fn main() {
    let msg = match do_it() {
        Ok(()) => format!("Success!"),
        Err(e) => format!("{}", e),
    };
    println!("{}", msg);
}

fn do_it() -> Result<(), Box<Error>> {
    sodiumoxide::init().map_err(|_| "sodiumoxide init failed")?;
    // TODO: better arg parsing, out parameter
    let argv = &std::env::args().collect::<Vec<String>>();
    if argv.len() < 3 { usage(); exit(1) }
    let mode = match argv[1].as_str() {
        "-e" => Mode::Encrypt,
        "-d" => Mode::Decrypt,
        _ => { usage(); exit(1) },
    };

    let filename = &argv[2];
    let p = Path::new(filename);
    if !(p.exists() && p.is_file()) {
        println!("Invalid filename: {}", filename);
        exit(1);
    }
    let out_filename = match mode {
        Mode::Encrypt => {
            let mut f = filename.to_string();
            f.push_str(FILE_EXTENSION);
            if Path::new(&f).exists() {
                f = find_filename(&f).ok_or("could not find filename")?;
            }
            f
        },
        Mode::Decrypt => {
            let mut f = if filename.ends_with(FILE_EXTENSION) {
                filename[..filename.len() - FILE_EXTENSION.len()].to_string()
            } else {
                match prepend("decrypted_", filename) {
                    Some(f) => f,
                    None => Err("oh no")?
                }
            };
            if Path::new(&f).exists() {
                f = find_filename(&f).ok_or("could not find filename")?;
            }
            f
        },
    };
    let password = rpassword::prompt_password_stdout("Password: ")?;
    let config = Config::new(mode, password, filename.to_string(), out_filename.to_string());
    main_routine(&config)
}

fn usage() {
    println!("Usage:\n$ ./cloaker -e secret.txt             (encrypting on Mac)\n> .\\cloaker.exe -d secret.txt.cloaker (decrypting on Windows)\n");
}

fn find_filename(name: &str) -> Option<String> {
    let mut i = 1;
    let backup_path = PathBuf::from(name);
    let mut path = backup_path.clone();
    while path.exists() {
        path = backup_path.clone();
        let stem = match path.file_stem() {
            Some(s) => s.to_string_lossy().to_string(),
            None => "".to_string(),
        };
        let ext = match path.extension() {
            Some(s) => s.to_string_lossy().to_string(),
            None => "".to_string(),
        };
        let parent = path.parent()?;
        let new_file = match ext.as_str() {
            "" => format!("{} ({})", stem, i),
            _ => format!("{} ({}).{}", stem, i, ext),
        };
        path = [
            parent,
            Path::new(&new_file),
        ].iter().collect();
        i += 1;
    }
    Some(path.to_string_lossy().to_string())
}

fn prepend(prefix: &str, p: &str) -> Option<String> {
    let mut path = PathBuf::from(p);
    let file = path.file_name()?;
    let parent = path.parent()?;
    path = [
        parent,
        Path::new(&format!("{}{}", prefix, file.to_string_lossy())),
    ].iter().collect();
    Some(path.to_string_lossy().to_string())
}
