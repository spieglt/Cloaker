
use std::error::Error;
use std::fs::{File, remove_file};
use std::io::prelude::*;
use sodiumoxide;

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
    pub ui: Box<dyn Ui>,
}

pub trait Ui {
    fn output(&self, percentage: i32);
}

impl Config {
    pub fn new(_mode: &Mode, password: String, _filename: Option<&str>, _out_file: Option<&str>, ui: Box<dyn Ui>) -> Self {
        let mode: Mode = _mode.clone();
        let filename = _filename.map(|f| f.to_string());
        let out_file = _out_file.map(|o| o.to_string());
        Config{mode, password, filename, out_file, ui}
    }
}

fn get_reader(reader: Option<File>) -> Box<dyn Read> {
    match reader {
        Some(file) => Box::new(file),
        None => Box::new(std::io::stdin()),
    }
}
fn get_writer(reader: Option<File>) -> Box<dyn Write> {
    match reader {
        Some(file) => Box::new(file),
        None => Box::new(std::io::stdout()),
    }
}

pub fn main_routine(c: &Config) -> Result<(), Box<dyn Error>> {
    sodiumoxide::init().map_err(|_| "sodiumoxide init failed")?;
    let in_file = match &c.filename {
        Some(s) => Some(File::open(s)?),
        None => None,
    };
    let out_file = match &c.out_file {
        Some(s) => Some(File::open(s)?),
        None => None,
    };
    let filesize = if let Some(f) = &in_file {
        Some(f.metadata()?.len() as usize)
    } else {
        None
    };

    let mut input = get_reader(in_file);
    let mut output = get_writer(out_file);
    match c.mode {
        Mode::Encrypt => {
            match crate::encrypt(&mut input, &mut output, &c.password, &c.ui, filesize) {
                Ok(()) => (),
                Err(e) => {
                    if let Some(out_file) = &c.out_file {
                        remove_file(&out_file).map_err(|e2| format!("{}. Could not delete output file: {}.", e, e2))?;
                    }
                    Err(e)?
                },
            };
        },
        Mode::Decrypt => {
            // start reading stream before handing to encrypt/decrypt
            // legacy decrypt might need a first-four-bytes param
            let mut first_four = [0u8; 4];
            if let Some(f) = &c.filename {
                let mut in_file = File::open(f)?;
                in_file.read_exact(&mut first_four)?;
            } else {
                std::io::stdin().read_exact(&mut first_four)?;
            }
            match first_four {
                crate::SIGNATURE => {
                    match crate::decrypt(&mut input, &mut output, &c.password, &c.ui, filesize) {
                        Ok(()) => (),
                        Err(e) => {
                            if let Some(out_file) = &c.out_file {
                                remove_file(&out_file).map_err(|e2| format!("{}. Could not delete output file: {}.", e, e2))?;
                            }
                            Err(e)?
                        },
                    };
                },
                _ => {
                    match crate::legacy::decrypt(&mut input, &mut output, &c.password, &c.ui, filesize) {
                        Ok(()) => (),
                        Err(e) => {
                            if let Some(out_file) = &c.out_file {
                                remove_file(&out_file).map_err(|e2| format!("{}. Could not delete output file: {}.", e, e2))?;
                            }
                            Err(e)?
                        },
                    };
                },
            }
        },
    }
    Ok(())
}
