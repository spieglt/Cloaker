//! End-to-end tests of the encrypt/decrypt entry point, including the legacy file formats
//! that Cloaker 4 still has to be able to read.

mod common;

use cloaker::Mode;
use common::*;
use std::fs::{read, write};

#[test]
fn round_trip_restores_the_original_bytes() {
    let dir = TempDir::new("round-trip");
    let plaintext = pseudorandom(4242, 1);
    let original = dir.file("original.bin");
    write(&original, &plaintext).unwrap();

    let encrypted = dir.file("original.bin.cloaker");
    run(Mode::Encrypt, PASSWORD, &original, &encrypted).unwrap();
    assert_ne!(read(&encrypted).unwrap(), plaintext);

    let decrypted = dir.file("decrypted.bin");
    run(Mode::Decrypt, PASSWORD, &encrypted, &decrypted).unwrap();
    assert_eq!(read(&decrypted).unwrap(), plaintext);
}

#[test]
fn round_trip_spanning_multiple_chunks() {
    let dir = TempDir::new("multi-chunk");
    // more than the 512KiB chunk size, and not a multiple of it
    let plaintext = pseudorandom(1024 * 700 + 13, 2);
    let original = dir.file("big.bin");
    write(&original, &plaintext).unwrap();

    let encrypted = dir.file("big.cloaker");
    let decrypted = dir.file("big.out");
    run(Mode::Encrypt, PASSWORD, &original, &encrypted).unwrap();
    run(Mode::Decrypt, PASSWORD, &encrypted, &decrypted).unwrap();
    assert_eq!(read(&decrypted).unwrap(), plaintext);
}

#[test]
fn round_trip_of_an_empty_file() {
    let dir = TempDir::new("empty");
    let original = dir.file("empty.bin");
    write(&original, b"").unwrap();

    let encrypted = dir.file("empty.cloaker");
    let decrypted = dir.file("empty.out");
    run(Mode::Encrypt, PASSWORD, &original, &encrypted).unwrap();
    run(Mode::Decrypt, PASSWORD, &encrypted, &decrypted).unwrap();
    assert_eq!(read(&decrypted).unwrap(), b"");
}

#[test]
fn progress_ends_at_one_hundred_percent() {
    let dir = TempDir::new("progress");
    let original = dir.file("in.bin");
    write(&original, pseudorandom(2048, 3)).unwrap();

    let encrypted = dir.file("in.cloaker");
    let recorder = ProgressRecorder::default();
    run_with_ui(
        Mode::Encrypt,
        PASSWORD,
        &original,
        &encrypted,
        Box::new(recorder.clone()),
    )
    .unwrap();

    let updates = recorder.updates.lock().unwrap();
    assert_eq!(updates.last(), Some(&100));
    assert!(updates.iter().all(|p| (0..=100).contains(p)));
}

#[test]
fn wrong_password_fails_and_removes_the_output_file() {
    let dir = TempDir::new("wrong-password");
    let original = dir.file("in.bin");
    write(&original, pseudorandom(1000, 4)).unwrap();
    let encrypted = dir.file("in.cloaker");
    run(Mode::Encrypt, PASSWORD, &original, &encrypted).unwrap();

    let decrypted = dir.file("out.bin");
    let err = run(Mode::Decrypt, "not the password", &encrypted, &decrypted).unwrap_err();
    assert!(
        err.contains("Incorrect password"),
        "unexpected error: {}",
        err
    );
    assert!(!decrypted.exists(), "failed decryption left a file behind");
}

#[test]
fn tampered_ciphertext_is_rejected() {
    let dir = TempDir::new("tampered");
    let original = dir.file("in.bin");
    write(&original, pseudorandom(1000, 5)).unwrap();
    let encrypted = dir.file("in.cloaker");
    run(Mode::Encrypt, PASSWORD, &original, &encrypted).unwrap();

    let mut bytes = read(&encrypted).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0xff;
    write(&encrypted, &bytes).unwrap();

    let decrypted = dir.file("out.bin");
    assert!(run(Mode::Decrypt, PASSWORD, &encrypted, &decrypted).is_err());
    assert!(!decrypted.exists());
}

#[test]
fn truncated_file_is_rejected_with_a_useful_message() {
    let dir = TempDir::new("truncated");
    let original = dir.file("in.bin");
    write(&original, pseudorandom(1024 * 1200, 6)).unwrap();
    let encrypted = dir.file("in.cloaker");
    run(Mode::Encrypt, PASSWORD, &original, &encrypted).unwrap();
    let bytes = read(&encrypted).unwrap();

    // signature + salt + header, then 512KiB chunks with a 17 byte authentication tag each
    let first_chunk_end = 4 + 32 + 24 + (512 * 1024 + 17);
    let decrypted = dir.file("out.bin");
    for cut in &[first_chunk_end, first_chunk_end + 1000] {
        // cut off the rest of the file, leaving a stream that never finalizes. the first chunk
        // still decrypts, so the password is known to be right: this has to be reported as damage.
        write(&encrypted, &bytes[..*cut]).unwrap();
        let err = run(Mode::Decrypt, PASSWORD, &encrypted, &decrypted).unwrap_err();
        assert!(err.contains("truncated"), "unexpected error: {}", err);
        assert!(!decrypted.exists());
    }
}

#[test]
fn file_too_small_to_be_encrypted_is_rejected() {
    let dir = TempDir::new("too-small");
    let input = dir.file("tiny.cloaker");
    let mut bytes = vec![0xC1, 0x0A, 0x6B, 0xED];
    bytes.extend_from_slice(b"nowhere near enough");
    write(&input, &bytes).unwrap();

    let output = dir.file("out.bin");
    let err = run(Mode::Decrypt, PASSWORD, &input, &output).unwrap_err();
    assert!(err.contains("not big enough"), "unexpected error: {}", err);
}

#[test]
fn unreadable_input_does_not_truncate_an_existing_output_file() {
    let dir = TempDir::new("no-truncate");
    let input = dir.file("two-bytes.bin");
    write(&input, b"hi").unwrap();
    let output = dir.file("precious.txt");
    write(&output, b"do not lose me").unwrap();

    assert!(run(Mode::Decrypt, PASSWORD, &input, &output).is_err());
    assert_eq!(read(&output).unwrap(), b"do not lose me");
}

#[test]
fn refuses_to_write_over_a_hard_link_to_its_own_input() {
    let dir = TempDir::new("hard-link");
    let file = dir.file("original.bin");
    write(&file, b"still here").unwrap();
    // a hard link is a second name for the same file, so its path resolves differently from the
    // input's even though writing to it would destroy the input
    let link = dir.file("another-name.bin");
    std::fs::hard_link(&file, &link).unwrap();

    let err = run(Mode::Encrypt, PASSWORD, &file, &link).unwrap_err();
    assert!(err.contains("same file"), "unexpected error: {}", err);
    assert_eq!(read(&file).unwrap(), b"still here");
}

#[cfg(unix)]
#[test]
fn refuses_to_write_over_a_symlink_to_its_own_input() {
    let dir = TempDir::new("symlink");
    let file = dir.file("original.bin");
    write(&file, b"still here").unwrap();
    let link = dir.file("pointer.bin");
    std::os::unix::fs::symlink(&file, &link).unwrap();

    let err = run(Mode::Encrypt, PASSWORD, &file, &link).unwrap_err();
    assert!(err.contains("same file"), "unexpected error: {}", err);
    assert_eq!(read(&file).unwrap(), b"still here");
}

#[test]
fn refuses_to_write_over_its_own_input() {
    let dir = TempDir::new("same-file");
    let file = dir.file("in-place.bin");
    write(&file, b"still here").unwrap();

    let err = run(Mode::Encrypt, PASSWORD, &file, &file).unwrap_err();
    assert!(err.contains("same file"), "unexpected error: {}", err);
    assert_eq!(read(&file).unwrap(), b"still here");
}

#[test]
fn decrypts_legacy_files_with_a_signature() {
    let dir = TempDir::new("legacy-signed");
    let plaintext = pseudorandom(LEGACY_CHUNKSIZE * 2 + 7, 7);
    let encrypted = dir.file("legacy.cloaker");
    write_legacy_file(&encrypted, PASSWORD, &plaintext, true);

    let decrypted = dir.file("legacy.out");
    run(Mode::Decrypt, PASSWORD, &encrypted, &decrypted).unwrap();
    assert_eq!(read(&decrypted).unwrap(), plaintext);
}

#[test]
fn decrypts_cloaker_1_0_files_without_a_signature() {
    let dir = TempDir::new("legacy-unsigned");
    let plaintext = pseudorandom(LEGACY_CHUNKSIZE + 1, 8);
    let encrypted = dir.file("ancient.cloaker");
    write_legacy_file(&encrypted, PASSWORD, &plaintext, false);

    let decrypted = dir.file("ancient.out");
    run(Mode::Decrypt, PASSWORD, &encrypted, &decrypted).unwrap();
    assert_eq!(read(&decrypted).unwrap(), plaintext);

    // and a wrong password against the same format still fails cleanly
    let bad_output = dir.file("ancient.bad");
    assert!(run(
        Mode::Decrypt,
        "some other password",
        &encrypted,
        &bad_output
    )
    .is_err());
    assert!(!bad_output.exists());
}
