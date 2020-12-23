mod brute_force; // test

use cloaker::*;

use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::exit;
use rpassword;
use clap::{App, Arg, ArgGroup};

const FILE_EXTENSION: &str = ".cloaker";

fn main() {
    let msg = match do_it() {
        Ok((output_filename, mode)) => {
            let m = match mode {
                Mode::Encrypt => "encrypted",
                Mode::Decrypt => "decrypted",
            };
            format!("Success! {} has been {}.", output_filename, m)
        },
        Err(e) => format!("{}", e),
    };
    println!("{}", msg);
}

fn do_it() -> Result<(String, Mode), Box<dyn Error>> {

    let matches = App::new("Cloaker")
        .version("v3.0")
        .author("Theron Spiegl")
        .about("Cloaker is a simple file encryption utility. Passwords must be at least 12 characters, and a variety of letters, numbers, and symbols is strongly recommended. Written in Rust using sodiumoxide/libsodium's stream encryption. Copyright © 2020 Theron Spiegl. All rights reserved.")
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
        .group(ArgGroup::with_name("mode")
            .args(&["encrypt", "decrypt"])
            .required(true))
        .arg(Arg::with_name("output")
            .short("o")
            .long("output")
            .value_name("PATH_TO_OUTPUT_FILE")
            .help("Specifies a path or name for the output file. If the path to an existing directory is given, the input filename will be kept with the .cloaker extension added if encrypting or removed (if decrypting). Otherwise the file will be placed and named according to this parameter."))
        .get_matches();

    let mode = if matches.is_present("encrypt") { Mode::Encrypt } else { Mode::Decrypt };
    let filename = match mode {
        Mode::Encrypt => matches.value_of("encrypt").ok_or("file to encrypt not given")?,
        Mode::Decrypt => matches.value_of("decrypt").ok_or("file to decrypt not given")?,
    };

    // make sure input file exists
    let p = Path::new(filename);
    if !(p.exists() && p.is_file()) {
        println!("Invalid filename: {}", filename);
        exit(1);
    }

    // build output path
    let output_path = generate_output_path(&mode, filename, matches.value_of("output"))?
        .to_str().ok_or("could not convert output path to string")?.to_string();
    let password = get_password(&mode);

    let config = Config::new(&mode, password, &filename, &output_path);
    match main_routine(&config) {
        Ok(()) => Ok((output_path.to_string(), mode)),
        Err(e) => Err(e),
    }
}

fn get_password(mode: &Mode) -> String {
    match mode {
        Mode::Encrypt => {
            let password = rpassword::prompt_password_stdout("Password (minimum 12 characters, longer is better): ")
                .expect("could not get password from user");
            if password.len() < 12 {
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

fn generate_output_path(mode: &Mode, input: &str, output: Option<&str>) -> Result<PathBuf, String> {
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

fn generate_default_filename(mode: &Mode, _path: PathBuf, name: &str) -> Result<PathBuf, String> {
    let mut path = _path;
    let f = match mode {
        Mode::Encrypt => {
            let mut with_ext = name.to_string();
            with_ext.push_str(FILE_EXTENSION);
            with_ext
        },
        Mode::Decrypt => {
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
