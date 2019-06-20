
use std::error::Error;
use std::fs::{File, remove_file};

#[derive(Debug)]
pub enum Mode {
    Encrypt,
    Decrypt,
}

#[derive(Debug)]
pub struct Config {
    pub mode: Mode,
    pub password: String,
    pub filename: String,
    pub out_file: String,
}

impl Config {
    pub fn new(mode: Mode, password: String, filename: String, out_file: String) -> Self {
        Config{mode, password, filename, out_file}
    }
}

pub fn main_routine(c: &Config) -> Result<(), Box<Error>> {
    let mut in_file = File::open(c.filename.clone())?;
    match c.mode {
        Mode::Encrypt => {
            let mut out_file = File::create(c.out_file.clone())?;
            match crate::encrypt(&mut in_file, &mut out_file, &c.password) {
                Ok(()) => (),
                Err(e) => {
                    remove_file(&c.out_file).map_err(|e2| format!("{}. Could not delete output file: {}.", e, e2))?;
                    Err(e)?;
                },
            };
        },
        Mode::Decrypt => {
            let mut out_file = File::create(c.out_file.clone())?;
            match crate::decrypt(&mut in_file, &mut out_file, &c.password) {
                Ok(()) => (),
                Err(e) => {
                    remove_file(&c.out_file).map_err(|e2| format!("{}. Could not delete output file: {}.", e, e2))?;
                    Err(e)?;
                },
            };
        },
    }
    Ok(())
}
