//! Proof that Cloaker's file format is exactly libsodium's, in both directions.
//!
//! Cloaker's own crypto is pure Rust (dryoc's secretstream, RustCrypto's argon2 and scrypt), so
//! these tests check it against libsodium itself rather than against Cloaker. Two kinds of check:
//! committed fixture files that were produced by libsodium once and must keep decrypting forever,
//! and round trips where one side is libsodium and the other is Cloaker.
//!
//! To regenerate the fixtures: `cargo test --test compatibility -- --ignored regenerate`

mod common;

use cloaker::Mode;
use common::*;
use std::fs::{read, write};
use std::path::{Path, PathBuf};

const FIXTURE_PASSWORD: &str = "fixtures are forever";

// (file, plaintext length, seed) — the plaintext is regenerated rather than committed
const V4_FIXTURE: (&str, usize, u64) = ("v4.cloaker", 4000, 0xC10A6BED);
const LEGACY_SIGNED_FIXTURE: (&str, usize, u64) = ("legacy_signed.cloaker", 10_000, 0xC10A4BED);
const LEGACY_UNSIGNED_FIXTURE: (&str, usize, u64) = ("legacy_1_0.cloaker", 5000, 0x10);

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn assert_fixture_decrypts((name, len, seed): (&str, usize, u64)) {
    let dir = TempDir::new("fixture");
    let out = dir.file("plaintext");
    run(Mode::Decrypt, FIXTURE_PASSWORD, &fixture(name), &out)
        .unwrap_or_else(|e| panic!("{name} no longer decrypts: {e}"));
    assert_eq!(
        read(&out).unwrap(),
        pseudorandom(len, seed),
        "{name} decrypted to the wrong bytes"
    );
}

#[test]
fn version_4_fixture_still_decrypts() {
    assert_fixture_decrypts(V4_FIXTURE);
}

#[test]
fn legacy_fixture_with_signature_still_decrypts() {
    assert_fixture_decrypts(LEGACY_SIGNED_FIXTURE);
}

#[test]
fn cloaker_1_0_fixture_still_decrypts() {
    assert_fixture_decrypts(LEGACY_UNSIGNED_FIXTURE);
}

#[test]
fn a_wrong_password_against_a_fixture_is_rejected() {
    let dir = TempDir::new("fixture-wrong-password");
    let out = dir.file("plaintext");
    let err = run(
        Mode::Decrypt,
        "not the fixture password",
        &fixture(V4_FIXTURE.0),
        &out,
    )
    .unwrap_err();
    assert!(
        err.contains("Incorrect password"),
        "unexpected error: {err}"
    );
    assert!(!out.exists());
}

#[test]
fn libsodium_reads_what_cloaker_writes() {
    let dir = TempDir::new("cloaker-to-libsodium");
    let plaintext = pseudorandom(9_000, 11);
    let original = dir.file("in.bin");
    write(&original, &plaintext).unwrap();
    let encrypted = dir.file("in.cloaker");

    run(Mode::Encrypt, PASSWORD, &original, &encrypted).unwrap();
    assert_eq!(
        reference_decrypt_v4(&encrypted, PASSWORD).unwrap(),
        plaintext,
        "libsodium could not read the file cloaker produced"
    );
    assert!(reference_decrypt_v4(&encrypted, "wrong password").is_err());
}

#[test]
fn libsodium_reads_what_cloaker_writes_across_several_chunks() {
    let dir = TempDir::new("cloaker-to-libsodium-chunks");
    // two full 512KiB chunks and a partial third
    let plaintext = pseudorandom(V4_CHUNKSIZE * 2 + 1234, 12);
    let original = dir.file("big.bin");
    write(&original, &plaintext).unwrap();
    let encrypted = dir.file("big.cloaker");

    run(Mode::Encrypt, PASSWORD, &original, &encrypted).unwrap();
    assert_eq!(
        reference_decrypt_v4(&encrypted, PASSWORD).unwrap(),
        plaintext
    );
}

#[test]
fn cloaker_reads_what_libsodium_writes() {
    let dir = TempDir::new("libsodium-to-cloaker");
    for (name, len) in [("small.cloaker", 1234usize), ("empty.cloaker", 0)] {
        let plaintext = pseudorandom(len, 13);
        let encrypted = dir.file(name);
        write_v4_file(&encrypted, PASSWORD, &plaintext);

        let decrypted = dir.file(&format!("{name}.out"));
        run(Mode::Decrypt, PASSWORD, &encrypted, &decrypted).unwrap();
        assert_eq!(read(&decrypted).unwrap(), plaintext, "failed for {name}");
    }
}

#[test]
fn cloaker_reads_what_libsodium_writes_across_several_chunks() {
    let dir = TempDir::new("libsodium-to-cloaker-chunks");
    let plaintext = pseudorandom(V4_CHUNKSIZE + 4321, 14);
    let encrypted = dir.file("big.cloaker");
    write_v4_file(&encrypted, PASSWORD, &plaintext);

    let decrypted = dir.file("big.out");
    run(Mode::Decrypt, PASSWORD, &encrypted, &decrypted).unwrap();
    assert_eq!(read(&decrypted).unwrap(), plaintext);
}

#[test]
#[ignore = "writes the committed fixture files; run explicitly to regenerate them"]
fn regenerate_fixtures() {
    let (name, len, seed) = V4_FIXTURE;
    write_v4_file(&fixture(name), FIXTURE_PASSWORD, &pseudorandom(len, seed));

    let (name, len, seed) = LEGACY_SIGNED_FIXTURE;
    write_legacy_file(
        &fixture(name),
        FIXTURE_PASSWORD,
        &pseudorandom(len, seed),
        true,
    );

    let (name, len, seed) = LEGACY_UNSIGNED_FIXTURE;
    write_legacy_file(
        &fixture(name),
        FIXTURE_PASSWORD,
        &pseudorandom(len, seed),
        false,
    );
    println!("fixtures written to {}", fixture("").display());
}
