mod brute_force; // test

use cloaker::*;

use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::exit;
use rpassword;
use clap::{App, Arg, ArgGroup};

const FILE_EXTENSION: &str = ".cloaker";

struct ProgressUpdater {
    mode: Mode,
    stdout: bool,
}

impl Ui for ProgressUpdater {
    fn output(&self, percentage: i32) {
        if !self.stdout {
            let s = match self.mode {
                Mode::Encrypt => "Encrypting",
                Mode::Decrypt => "Decrypting",
            };
            print!("\r{}: {}%", s, percentage);
        }
    }
}

fn main() {
    match do_it() {
        Ok((output_filename, mode)) => {
            let m = match mode {
                Mode::Encrypt => "encrypted",
                Mode::Decrypt => "decrypted",
            };
            if let Some(name) = output_filename {
                println!("\nSuccess! {} has been {}.", name, m);
            }
        },
        Err(e) => eprintln!("\n{}", e),
    };
}

fn do_it() -> Result<(Option<String>, Mode), Box<dyn Error>> {
    let matches = App::new("Cloaker")
        .version("v4.0")
        .author("Theron Spiegl")
        .about("Cloaker is a simple file encryption utility. Passwords must be at least 12 characters, and a variety of letters, numbers, and symbols is strongly recommended. Written in Rust using sodiumoxide/libsodium's stream encryption. Copyright © 2020 Theron Spiegl. All rights reserved. https://spiegl.dev/cloaker")
        .arg(Arg::with_name("encrypt")
            .short("e")
            .long("encrypt")
            .value_name("FILE_TO_ENCRYPT")
            .help("Specifies the file to encrypt.")
            .takes_value(true))
        .arg(Arg::with_name("decrypt")
            .short("d")
            .long("decrypt")
            .value_name("FILE_TO_DECRYPT")
            .help("Specifies the file to decrypt.")
            .takes_value(true))
        .arg(Arg::with_name("encrypt_stdin")
            .short("E")
            .long("encrypt-stdin")
            .help("Encrypt from stdin instead of a file.")
            .requires("password_flags"))
        .arg(Arg::with_name("decrypt_stdin")
            .short("D")
            .long("decrypt-stdin")
            .help("Decrypt from stdin instead of a file.")
            .requires("password_flags"))
        .group(ArgGroup::with_name("mode")
            .args(&["encrypt", "decrypt", "encrypt_stdin", "decrypt_stdin"])
            .required(true))
        .arg(Arg::with_name("output")
            .short("o")
            .long("output")
            .value_name("PATH_TO_OUTPUT_FILE")
            .help("Specifies a path or name for the output file. If the path to an existing directory is given, the input filename will be kept with the .cloaker extension added if encrypting or removed (if decrypting). Otherwise the file will be placed and named according to this parameter.")
            .takes_value(true))
        .arg(Arg::with_name("stdout")
            .short("O")
            .long("stdout")
            .help("Encrypt to stdout instead of to a file.")
            .requires("password_flags"))
        .group(ArgGroup::with_name("destination")
            .args(&["output", "stdout"]))
        .arg(Arg::with_name("password")
            .short("p")
            .long("password")
            .value_name("PASSWORD")
            .help("Optional, and not recommended due to the password appearing in shell history. Password for the file. This or the --password-file (-f) flag is required if using stdin and/or stdout.")
            .takes_value(true))
        .arg(Arg::with_name("password_file")
            .short("f")
            .long("password-file")
            .value_name("PASSWORD_FILE")
            .help("The password to encrypt/decrypt with will be read from a text file at the path provided. File should be valid UTF-8 and contain only the password with no newline. This or the --password (-p) flag is required if using stdin and/or stdout.")
            .takes_value(true))
        .group(ArgGroup::with_name("password_flags")
            .args(&["password", "password_file"]))
        .get_matches();

    let mode = if matches.is_present("encrypt") || matches.is_present("encrypt_stdin") {
        Mode::Encrypt
    } else {
        Mode::Decrypt
    };

    let filename = if matches.is_present("encrypt") {
        let f = matches.value_of("encrypt").ok_or("file to encrypt not given")?;
        // make sure input file exists
        let p = Path::new(f);
        if !(p.exists() && p.is_file()) {
            println!("Invalid filename: {}", f);
            exit(1);
        }
        Some(f)
    } else if matches.is_present("decrypt") {
        let f = matches.value_of("decrypt").ok_or("file to decrypt not given")?;
        let p = Path::new(f);
        if !(p.exists() && p.is_file()) {
            println!("Invalid filename: {}", f);
            exit(1);
        }
        Some(f)
    } else {
        None // using stdin
    };


    let output_path = if !matches.is_present("stdout") {
        let s = generate_output_path(&mode, filename, matches.value_of("output"))?
            .to_str().ok_or("could not convert output path to string")?.to_string();
        Some(s)
    } else {
        None
    };

    // get_password needs to only happen if using neither stdin nor stdout: using requires() in clap
    // password prompting is affected by both stdin and stdout, whereas other printing is affected only by stdout
    let password = if matches.is_present("password") {
        matches.value_of("password").ok_or("couldn't get password value")?.to_string()
    } else if matches.is_present("password_file") {
        let pw_file = matches.value_of("password_file").ok_or("could not get value of password file")?.to_string();
        let p = Path::new(&pw_file);
        std::fs::read_to_string(p).map_err(|e| format!("could not read password file: {}", e.to_string()))?
    } else {
        get_password(&mode)
    };
    let ui = Box::new(ProgressUpdater{mode: mode.clone(), stdout: matches.is_present("stdout")});
    let config = Config::new(&mode, password, filename.map(|f| f.to_string()), output_path.clone(), ui);
    match main_routine(&config) {
        Ok(()) => Ok((output_path, mode)),
        Err(e) => Err(e),
    }
}

fn get_password(mode: &Mode) -> String {
    match mode {
        Mode::Encrypt => {
            let password = rpassword::prompt_password_stdout("Password (minimum 12 characters, longer is better): ")
                .expect("could not get password from user");
            if password.len() > 12 {
                println!("Error: password must be at least 12 characters. Exiting.");
                exit(12);
            }
            let verified_password = rpassword::prompt_password_stdout("Confirm password: ")
                .expect("could not get password from user");
            if password != verified_password {
                println!("Error: passwords do not match. Exiting.");
                exit(1);
            }
            password
        },
        Mode::Decrypt => rpassword::prompt_password_stdout("Password: ").expect("could not get password from user"),
    }
}

fn generate_output_path(mode: &Mode, input: Option<&str>, output: Option<&str>) -> Result<PathBuf, String> {
    if output.is_some() { // if output flag was specified,
        let p = PathBuf::from(output.unwrap());
        if p.exists() && p.is_dir() { // and it's a directory,
            generate_default_filename(mode, p, input) // give it a default filename.
        } else if p.exists() && p.is_file() {
            Err(format!("Error: file {:?} already exists. Must choose new filename or specify directory to generate default filename.", p))
        } else { // otherwise use it as the output filename.
            Ok(p)
        }
    } else { // if output not specified, generate default filename and put in the current working directory
        let cwd = env::current_dir().map_err(|e| e.to_string())?;
        generate_default_filename(mode, cwd, input)
    }
}

fn generate_default_filename(mode: &Mode, _path: PathBuf, name: Option<&str>) -> Result<PathBuf, String> {
    let mut path = _path;
    let f = match mode {
        Mode::Encrypt => {
            let mut with_ext = if let Some(n) = name { n.to_string() } else { "encrypted".to_string() };
            with_ext.push_str(FILE_EXTENSION);
            with_ext
        },
        Mode::Decrypt => {
            let name = if let Some(n) = name { n } else { "stdin" };
            if name.ends_with(FILE_EXTENSION) {
                name[..name.len() - FILE_EXTENSION.len()].to_string()
            } else {
                prepend("decrypted_", name).ok_or(format!("could not prepend decrypted_ to filename {}", name))?
            }
        },
    };
    path.push(f);
    find_filename(path).ok_or("could not generate filename".to_string())
}

fn find_filename(_path: PathBuf) -> Option<PathBuf> {
    let mut i = 1;
    let mut path = _path;
    let backup_path = path.clone();
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
    Some(path)
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
