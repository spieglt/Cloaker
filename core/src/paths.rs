//! Output path and password rules shared by the CLI and the GUI, so the two can't drift.

use std::path::{Path, PathBuf};

pub const FILE_EXTENSION: &str = ".cloaker";
pub const MIN_PASSWORD_LENGTH: usize = 12;

use crate::Mode;

pub fn check_password_length(password: &str) -> Result<(), String> {
    if password.chars().count() < MIN_PASSWORD_LENGTH {
        return Err(format!(
            "Error: password must be at least {} characters.",
            MIN_PASSWORD_LENGTH
        ));
    }
    Ok(())
}

pub fn generate_output_path(
    mode: &Mode,
    input: Option<&str>,
    output: Option<&str>,
) -> Result<PathBuf, String> {
    match output {
        // if output flag was specified,
        Some(o) => {
            let p = PathBuf::from(o);
            if p.is_dir() {
                // and it's a directory,
                generate_default_filename(mode, p, input) // give it a default filename.
            } else if p.is_file() {
                Err(format!("Error: file {:?} already exists. Must choose new filename or specify directory to generate default filename.", p))
            } else {
                // otherwise use it as the output filename.
                Ok(p)
            }
        }
        // if output not specified, generate default filename and put in the current working directory
        None => {
            let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
            generate_default_filename(mode, cwd, input)
        }
    }
}

// builds a filename from the input's name and puts it in `dir`. only the input's filename is
// used, so that an input path elsewhere on disk doesn't drag the output along with it.
pub fn generate_default_filename(
    mode: &Mode,
    dir: PathBuf,
    input: Option<&str>,
) -> Result<PathBuf, String> {
    let mut path = dir;
    let name = match input {
        Some(i) => Some(
            Path::new(i)
                .file_name()
                .ok_or_else(|| format!("could not determine filename of {}", i))?
                .to_string_lossy()
                .to_string(),
        ),
        None => None,
    };
    let f = match mode {
        Mode::Encrypt => format!(
            "{}{}",
            name.as_deref().unwrap_or("encrypted"),
            FILE_EXTENSION
        ),
        Mode::Decrypt => {
            let name = name.as_deref().unwrap_or("stdin");
            match name.strip_suffix(FILE_EXTENSION) {
                Some(stripped) if !stripped.is_empty() => stripped.to_string(),
                _ => format!("decrypted_{}", name),
            }
        }
    };
    path.push(f);
    find_filename(path).ok_or_else(|| "could not generate filename".to_string())
}

// adds " (1)", " (2)" and so on until the name is free, so an existing file is never clobbered
pub fn find_filename(_path: PathBuf) -> Option<PathBuf> {
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
        path = [parent, Path::new(&new_file)].iter().collect();
        i += 1;
    }
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_length_is_counted_in_characters() {
        assert!(check_password_length("hunter2").is_err());
        assert!(check_password_length("123456789012").is_ok());
        // 11 multi-byte characters are long enough to pass a byte-count check, but are too short
        assert!(check_password_length("ααααααααααα").is_err());
        assert!(check_password_length("αααααααααααα").is_ok());
    }

    #[test]
    fn default_filename_goes_in_the_given_directory() {
        let dir = std::env::temp_dir();
        let encrypted =
            generate_default_filename(&Mode::Encrypt, dir.clone(), Some("/etc/hosts")).unwrap();
        assert_eq!(encrypted, dir.join("hosts.cloaker"));

        let decrypted = generate_default_filename(
            &Mode::Decrypt,
            dir.clone(),
            Some("/some/other/place/notes.txt.cloaker"),
        )
        .unwrap();
        assert_eq!(decrypted, dir.join("notes.txt"));

        let no_extension =
            generate_default_filename(&Mode::Decrypt, dir.clone(), Some("sub/dir/secret")).unwrap();
        assert_eq!(no_extension, dir.join("decrypted_secret"));
    }

    #[test]
    fn default_filename_without_input_uses_stdin_names() {
        let dir = std::env::temp_dir();
        assert_eq!(
            generate_default_filename(&Mode::Encrypt, dir.clone(), None).unwrap(),
            dir.join("encrypted.cloaker")
        );
        assert_eq!(
            generate_default_filename(&Mode::Decrypt, dir.clone(), None).unwrap(),
            dir.join("decrypted_stdin")
        );
    }

    #[test]
    fn existing_output_file_is_never_overwritten() {
        let dir = std::env::temp_dir().join(format!("cloaker-names-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("notes.txt.cloaker"), b"taken").unwrap();

        let generated =
            generate_default_filename(&Mode::Encrypt, dir.clone(), Some("notes.txt")).unwrap();
        assert_eq!(generated, dir.join("notes.txt (1).cloaker"));

        // an output path that names an existing file is rejected rather than clobbered
        let existing = dir.join("notes.txt.cloaker");
        let err = generate_output_path(&Mode::Encrypt, Some("notes.txt"), existing.to_str());
        assert!(err.is_err());

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
