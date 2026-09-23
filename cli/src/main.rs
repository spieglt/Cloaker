#[cfg(test)]
mod brute_force;

use cloaker::*;

use clap::{App, Arg, ArgGroup};
use std::error::Error;
use std::io::Write;
use std::path::Path;
use std::process::exit;

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
            // progress has no trailing newline, so it needs flushing to be visible
            let _ = std::io::stdout().flush();
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
        }
        Err(e) => {
            eprintln!("\n{}", e);
            exit(1);
        }
    };
}

fn do_it() -> Result<(Option<String>, Mode), Box<dyn Error>> {
    let matches = App::new("Cloaker")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Cloaker is a simple file encryption utility. Passwords must be at least 12 characters, though longer is better. Written in pure Rust, using libsodium's XChaCha20-Poly1305 secretstream format. Copyright © 2026 Theron Spiegl. All rights reserved. https://cloaker.spiegl.dev/")
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
            .help("Encrypt from stdin instead of a file. If an output filename is not specified with -o, the default will be `./encrypted.cloaker`.")
            .requires("password_flags"))
        .arg(Arg::with_name("decrypt_stdin")
            .short("D")
            .long("decrypt-stdin")
            .help("Decrypt from stdin instead of a file. If an output filename is not specified with -o, the default will be `./decrypted_stdin`.")
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
            .help("Encrypt or decrypt to stdout instead of to a file.")
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
            .help("The password to encrypt/decrypt with will be read from a text file at the path provided. File should be valid UTF-8 and contain only the password. A single trailing newline, if present, is ignored. This or the --password (-p) flag is required if using stdin and/or stdout.")
            .takes_value(true))
        .group(ArgGroup::with_name("password_flags")
            .args(&["password", "password_file"]))
        .get_matches();

    let mode = if matches.is_present("encrypt") || matches.is_present("encrypt_stdin") {
        Mode::Encrypt
    } else {
        Mode::Decrypt
    };

    let filename = match (matches.value_of("encrypt"), matches.value_of("decrypt")) {
        (Some(f), _) | (_, Some(f)) => {
            // make sure input file exists
            if !Path::new(f).is_file() {
                return Err(format!("Invalid filename: {}", f).into());
            }
            Some(f)
        }
        _ => None, // using stdin
    };

    let output_path = if !matches.is_present("stdout") {
        let s = generate_output_path(&mode, filename, matches.value_of("output"))?
            .to_str()
            .ok_or("could not convert output path to string")?
            .to_string();
        Some(s)
    } else {
        None
    };

    // get_password needs to only happen if using neither stdin nor stdout: using requires() in clap
    // password prompting is affected by both stdin and stdout, whereas other printing is affected only by stdout
    let mut unstripped_password = None;
    let password = if let Some(p) = matches.value_of("password") {
        p.to_string()
    } else if let Some(pw_file) = matches.value_of("password_file") {
        let contents = std::fs::read_to_string(Path::new(pw_file))
            .map_err(|e| format!("could not read password file: {}", e))?;
        let stripped = strip_trailing_newline(&contents);
        if stripped != contents {
            // earlier versions used the file verbatim, newline included, so keep it as a fallback
            unstripped_password = Some(contents.clone());
        }
        stripped.to_string()
    } else {
        get_password(&mode)?
    };
    // the minimum length applies no matter where the password came from
    if let Mode::Encrypt = mode {
        check_password_length(&password)?;
    }

    let to_stdout = matches.is_present("stdout");
    let result = run(
        &mode,
        &password,
        filename,
        output_path.as_deref(),
        to_stdout,
    );
    match result {
        Ok(()) => Ok((output_path, mode)),
        Err(e) => {
            // a file encrypted by an older version may have included the password file's trailing
            // newline in the password. retry with it, but only when both ends are real files:
            // stdin can't be read twice, and stdout may already have been written to.
            let retryable = matches!(mode, Mode::Decrypt) && filename.is_some() && !to_stdout;
            if let (true, Some(unstripped)) = (retryable, &unstripped_password) {
                if run(
                    &mode,
                    unstripped,
                    filename,
                    output_path.as_deref(),
                    to_stdout,
                )
                .is_ok()
                {
                    return Ok((output_path, mode));
                }
            }
            Err(e)
        }
    }
}

fn run(
    mode: &Mode,
    password: &str,
    filename: Option<&str>,
    output_path: Option<&str>,
    to_stdout: bool,
) -> Result<(), Box<dyn Error>> {
    let ui = Box::new(ProgressUpdater {
        mode: mode.clone(),
        stdout: to_stdout,
    });
    let config = Config::new(
        mode,
        password.to_string(),
        filename.map(|f| f.to_string()),
        output_path.map(|o| o.to_string()),
        ui,
    );
    main_routine(&config)
}

// a password file written with `echo hunter2... > pw.txt` ends in a newline that the user
// doesn't consider part of their password, so ignore one trailing line ending
fn strip_trailing_newline(contents: &str) -> &str {
    contents
        .strip_suffix('\n')
        .map(|s| s.strip_suffix('\r').unwrap_or(s))
        .unwrap_or(contents)
}

fn get_password(mode: &Mode) -> Result<String, Box<dyn Error>> {
    match mode {
        Mode::Encrypt => {
            let password = rpassword::prompt_password(format!(
                "Password (minimum {} characters, longer is better): ",
                MIN_PASSWORD_LENGTH
            ))?;
            check_password_length(&password)?;
            let verified_password = rpassword::prompt_password("Confirm password: ")?;
            if password != verified_password {
                return Err("Error: passwords do not match.".into());
            }
            Ok(password)
        }
        Mode::Decrypt => Ok(rpassword::prompt_password("Password: ")?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_one_trailing_line_ending() {
        assert_eq!(strip_trailing_newline("password"), "password");
        assert_eq!(strip_trailing_newline("password\n"), "password");
        assert_eq!(strip_trailing_newline("password\r\n"), "password");
        assert_eq!(strip_trailing_newline("password\n\n"), "password\n");
        // whitespace other than the final line ending belongs to the password
        assert_eq!(strip_trailing_newline(" pass word \n"), " pass word ");
    }
}
