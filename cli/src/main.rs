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

fn do_it() -> Result<(), Box<dyn Error>> {
    sodiumoxide::init().map_err(|_| "sodiumoxide init failed")?;
    // TODO: better arg parsing, out parameter, password minimum, password verification
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
    let config = Config::new(mode, password, &filename, &out_filename);
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

#[cfg(test)]
mod tests {
    use rand::{thread_rng, RngCore};
    use std::io::Write;
    use itertools::Itertools;
    use std::time::Instant;

    #[test]
    fn brute_force_test() -> Result<(), Box<dyn std::error::Error>> {

        // generate random file, write to temp location
        let mut random_data = vec![0; (1<<10) * 100]; // 100KiB
        thread_rng().fill_bytes(&mut random_data);
        let mut temp_file = std::env::temp_dir();
        temp_file.push("rand.txt");
        let mut file = std::fs::File::create(&temp_file)?;
        file.write_all(&random_data)?;

        // encrypt file with 10-char password
        let pw = "abcdefghij".to_string();
        let in_file = temp_file.to_str().unwrap().to_string();
        let mut out_path = std::env::temp_dir();
        out_path.push("encrypted.txt");
        let out_file = out_path.to_str().unwrap().to_string();
        let config = cloaker::Config::new(cloaker::Mode::Encrypt, pw, &in_file, &out_file);
        cloaker::main_routine(&config)?;

        // measure frequency of brute-force attempts
        /*
            Uppercase and lowercase letters makes 52 values. With a 10 character minimum, that makes 90,177,170,226 passwords.
            Letters, numbers, and symbols makes 94 values. With a 10 character minimum, that makes 23,591,276,125,340 passwords.
        */

        let mut possible_chars = ('a'..='z').collect::<Vec<char>>();
        possible_chars.append(&mut ('A'..='Z').collect());
        let mut nums_and_syms = "0123456789!@#$%^&*()-_=+`~,./<>?;':\"[]{}\\|".as_bytes().to_vec();
        possible_chars.append(&mut "0123456789!@#$%^&*()-_=+`~,./<>?;':\"[]{}\\|".chars().collect());
        let mut combiner = possible_chars.iter().combinations_with_replacement(10);
        println!("possible chars: {}", possible_chars.len());

        let num_combos = 121623751733457400.;
        let mut start_time = Instant::now();
        let mut attempts = 0;

        loop {
            let guess_chars = combiner
                .next() // get next combination from the iterator, which will be a Vec<&char>
                .ok_or("end of combinations")? // coerce None to Err so we can fit the surrounding function signature and use the question mark
                .iter() // have to iterate over it so we can...
                .cloned().cloned() // clone it twice, which is weird. we're dealing with references to references at this point I guess so have to undo it twice.
                .collect::<Vec<char>>(); // and then collect it into a vector of chars.
            let guess: String = guess_chars.into_iter().collect();
            // println!("guess: {}", guess);
            let c = cloaker::Config::new(cloaker::Mode::Decrypt, guess, &out_file, "./result");
            assert!(cloaker::main_routine(&c).is_err());

            attempts += 1;
            let elapsed = Instant::now().duration_since(start_time.clone()).as_secs_f64();
            if elapsed == 0. {continue};
            let attempts_per_sec = attempts as f64 / elapsed;
            // attempts_per_sec * num_secs = num_combos, so num_secs = num_combos / attempts_per_sec
            let num_secs = num_combos / attempts_per_sec;
            let num_years = num_secs / (60. * 60. * 24. * 365.);
            println!("at {:.3} attempts per second, it would take {:.2} years to test all 10-character passwords including lower-/uppercase letters, numbers, and symbols.", attempts_per_sec, num_years);
        }
        Ok(())
    }
}
