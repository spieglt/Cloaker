use cloaker::*;

use std::error::Error;
use std::path::Path;
use std::process::exit;
use rpassword;
use sodiumoxide;

fn main() {
    let msg = match do_it() {
        Ok(s) => format!("Success! File {} is alongside the original.", s),
        Err(e) => format!("{}", e),
    };
    println!("{}", msg);
}

fn do_it() -> Result<String, Box<Error>> {
    sodiumoxide::init().map_err(|_| "sodiumoxide init failed")?;
    // TODO: better arg parsing
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
    let password = rpassword::read_password_from_tty(Some("Password: "))?;
    let config = Config::new(mode, password, filename.to_string());
    main_routine(&config)
}

fn usage() {
    println!("Usage:\n$ ./cloaker -e secret.txt             (encrypting on Mac)\n> .\\cloaker.exe -d secret.txt.cloaker (decrypting on Windows)\n");
}
