//! A bounded version of the brute-force experiment: it encrypts a file, makes a handful of
//! wrong-password attempts against it, asserts that every attempt fails and leaves no output
//! behind, and prints how long an exhaustive search would take at the measured rate.

use std::fs::{create_dir_all, remove_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

// letters, numbers, and symbols make 94 values
const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-_=+`~,./<>?;':\"[]{}\\|";
const ATTEMPTS: u32 = 5;

struct ProgressUpdater {}

impl cloaker::Ui for ProgressUpdater {
    fn output(&self, _percentage: i32) {}
}

// a scratch directory that cleans up after itself
struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("cloaker-{}-{}", name, std::process::id()));
        let _ = remove_dir_all(&path);
        create_dir_all(&path).expect("could not create temp directory");
        TempDir { path }
    }

    fn file(&self, name: &str) -> String {
        self.path.join(name).to_string_lossy().to_string()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = remove_dir_all(&self.path);
    }
}

// deterministic stand-in for random data, so a failure is always reproducible
fn pseudorandom(len: usize, seed: u64) -> Vec<u8> {
    let mut state = seed | 1;
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 24) as u8
        })
        .collect()
}

// the nth password of ALPHABET, counting in base 94
fn nth_guess(mut n: u64, length: usize) -> String {
    let base = ALPHABET.len() as u64;
    let mut chars = Vec::with_capacity(length);
    for _ in 0..length {
        chars.push(ALPHABET[(n % base) as usize]);
        n /= base;
    }
    String::from_utf8(chars).expect("alphabet is ascii")
}

#[test]
fn wrong_passwords_are_rejected_at_an_infeasible_rate() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new("brute-force");

    // generate a file and encrypt it with a 12-character password
    let in_file = dir.file("rand.bin");
    File::create(&in_file)?.write_all(&pseudorandom((1 << 10) * 100, 0x5c10a6bed))?; // 100KiB
    let encrypted = dir.file("encrypted.cloaker");
    let config = cloaker::Config::new(
        &cloaker::Mode::Encrypt,
        "abcdefghijkl".to_string(),
        Some(in_file),
        Some(encrypted.clone()),
        Box::new(ProgressUpdater {}),
    );
    cloaker::main_routine(&config)?;

    // measure how many guesses per second an attacker with this machine would get
    let guessed = dir.file("guessed");
    let start_time = Instant::now();
    for attempt in 0..ATTEMPTS {
        let guess = nth_guess(attempt as u64, 12);
        let c = cloaker::Config::new(
            &cloaker::Mode::Decrypt,
            guess.clone(),
            Some(encrypted.clone()),
            Some(guessed.clone()),
            Box::new(ProgressUpdater {}),
        );
        assert!(
            cloaker::main_routine(&c).is_err(),
            "guess {} should not have decrypted the file",
            guess
        );
        assert!(
            !PathBuf::from(&guessed).exists(),
            "failed attempt left an output file behind"
        );
    }
    let elapsed = start_time.elapsed().as_secs_f64();

    // 94 possible characters and a 12 character minimum
    let num_combos = (ALPHABET.len() as f64).powi(12);
    let attempts_per_sec = ATTEMPTS as f64 / elapsed.max(f64::EPSILON);
    let num_years = num_combos / attempts_per_sec / (60. * 60. * 24. * 365.);
    println!(
        "at {:.3} attempts per second, it would take {:.2e} years to test all {}-character \
         passwords including lower-/uppercase letters, numbers, and symbols.",
        attempts_per_sec, num_years, 12
    );

    Ok(())
}
