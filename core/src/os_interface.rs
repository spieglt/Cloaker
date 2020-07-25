
use std::error::Error;
use std::fs::{File, remove_file};

#[derive(Clone, Debug)]
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
    pub fn new(_mode: &Mode, password: String, _filename: &str, _out_file: &str) -> Self {
        let mode: Mode = _mode.clone();
        let filename = _filename.to_string();
        let out_file = _out_file.to_string();
        Config{mode, password, filename, out_file}
    }
}

pub fn main_routine(c: &Config) -> Result<(), Box<dyn Error>> {
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
