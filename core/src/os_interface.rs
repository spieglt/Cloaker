
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

// fn find_filename(name: &String) -> Option<String> {
//     let mut i = 1;
//     let backup_path = PathBuf::from(name);
//     let mut path = backup_path.clone();
//     while path.exists() {
//         path = backup_path.clone();
//         let stem = match path.file_stem() {
//             Some(s) => s.to_string_lossy().to_string(),
//             None => "".to_string(),
//         };
//         let ext = match path.extension() {
//             Some(s) => s.to_string_lossy().to_string(),
//             None => "".to_string(),
//         };
//         let parent = path.parent()?;
//         let new_file = match ext.as_str() {
//             "" => format!("{} ({})", stem, i),
//             _ => format!("{} ({}).{}", stem, i, ext),
//         };
//         path = [
//             parent,
//             Path::new(&new_file),
//         ].iter().collect();
//         i += 1;
//     }
//     Some(path.to_string_lossy().to_string())
// }

// fn prepend(prefix: &str, p: &str) -> Option<String> {
//     let mut path = PathBuf::from(p);
//     let file = path.file_name()?;
//     let parent = path.parent()?;
//     path = [
//         parent,
//         Path::new(&format!("{}{}", prefix, file.to_string_lossy())),
//     ].iter().collect();
//     Some(path.to_string_lossy().to_string())
// }
