//! Test helpers shared by the integration tests, including reference encoders and decoders built
//! directly on libsodium (via sodiumoxide). Those are what keep the file format honest: Cloaker's
//! own crypto is pure Rust, so the tests check it against the reference implementation rather than
//! against itself.
#![allow(dead_code)]

use cloaker::{main_routine, Config, Mode, Ui};
use sodiumoxide::crypto::pwhash;
use sodiumoxide::crypto::pwhash::argon2id13;
use sodiumoxide::crypto::secretstream::xchacha20poly1305::{
    Header, Key, Stream, Tag, ABYTES, HEADERBYTES, KEYBYTES,
};
use std::fs::{create_dir_all, read, remove_dir_all, write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

pub const PASSWORD: &str = "correct horse battery staple";
pub const LEGACY_SIGNATURE: [u8; 4] = [0xC1, 0x0A, 0x4B, 0xED];
pub const LEGACY_CHUNKSIZE: usize = 4096;
pub const V4_SIGNATURE: [u8; 4] = [0xC1, 0x0A, 0x6B, 0xED];
pub const V4_CHUNKSIZE: usize = 1024 * 512;

// a scratch directory that cleans up after itself
pub struct TempDir {
    pub path: PathBuf,
}

impl TempDir {
    pub fn new(name: &str) -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "cloaker-{}-{}-{}",
            name,
            std::process::id(),
            unique
        ));
        let _ = remove_dir_all(&path);
        create_dir_all(&path).expect("could not create temp directory");
        TempDir { path }
    }

    pub fn file(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = remove_dir_all(&self.path);
    }
}

#[derive(Clone, Default)]
pub struct ProgressRecorder {
    pub updates: Arc<Mutex<Vec<i32>>>,
}

impl Ui for ProgressRecorder {
    fn output(&self, percentage: i32) {
        self.updates.lock().unwrap().push(percentage);
    }
}

pub fn path_string(p: &Path) -> String {
    p.to_string_lossy().to_string()
}

pub fn run(mode: Mode, password: &str, input: &Path, output: &Path) -> Result<(), String> {
    run_with_ui(
        mode,
        password,
        input,
        output,
        Box::new(ProgressRecorder::default()),
    )
}

pub fn run_with_ui(
    mode: Mode,
    password: &str,
    input: &Path,
    output: &Path,
    ui: Box<dyn Ui + Send>,
) -> Result<(), String> {
    let config = Config::new(
        &mode,
        password.to_string(),
        Some(path_string(input)),
        Some(path_string(output)),
        ui,
    );
    main_routine(&config).map_err(|e| e.to_string())
}

// deterministic stand-in for random data, so a failure is always reproducible
pub fn pseudorandom(len: usize, seed: u64) -> Vec<u8> {
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

// writes a file in the format used by older versions of Cloaker: 4096-byte chunks keyed with
// argon2i, optionally preceded by the legacy signature (version 1.0 files have no signature).
pub fn write_legacy_file(path: &Path, password: &str, plaintext: &[u8], with_signature: bool) {
    sodiumoxide::init().unwrap();
    let salt = pwhash::gen_salt();
    let mut key_bytes = [0u8; KEYBYTES];
    pwhash::derive_key(
        &mut key_bytes,
        password.as_bytes(),
        &salt,
        pwhash::OPSLIMIT_INTERACTIVE,
        pwhash::MEMLIMIT_INTERACTIVE,
    )
    .unwrap();
    let key = Key(key_bytes);
    let (mut stream, header) = Stream::init_push(&key).unwrap();

    let mut out = Vec::new();
    if with_signature {
        out.extend_from_slice(&LEGACY_SIGNATURE);
    }
    out.extend_from_slice(&salt.0);
    out.extend_from_slice(&header.0);
    let mut start = 0;
    loop {
        let end = std::cmp::min(start + LEGACY_CHUNKSIZE, plaintext.len());
        let last = end == plaintext.len();
        let tag = if last { Tag::Final } else { Tag::Message };
        out.extend_from_slice(&stream.push(&plaintext[start..end], None, tag).unwrap());
        start = end;
        if last {
            break;
        }
    }
    write(path, out).unwrap();
}

// writes a file in the current (version 4) format using libsodium directly: argon2id13 for the key
// and the secretstream for the data, in 512KiB chunks.
pub fn write_v4_file(path: &Path, password: &str, plaintext: &[u8]) {
    sodiumoxide::init().unwrap();
    let salt = argon2id13::gen_salt();
    let mut key_bytes = [0u8; KEYBYTES];
    argon2id13::derive_key(
        &mut key_bytes,
        password.as_bytes(),
        &salt,
        argon2id13::OPSLIMIT_INTERACTIVE,
        argon2id13::MEMLIMIT_INTERACTIVE,
    )
    .unwrap();
    let key = Key(key_bytes);
    let (mut stream, header) = Stream::init_push(&key).unwrap();

    let mut out = Vec::new();
    out.extend_from_slice(&V4_SIGNATURE);
    out.extend_from_slice(&salt.0);
    out.extend_from_slice(&header.0);
    out.extend_from_slice(&push_chunks(&mut stream, plaintext, V4_CHUNKSIZE));
    write(path, out).unwrap();
}

// mirrors cloaker's own loop: a plaintext that is an exact multiple of the chunk size is followed
// by an empty final chunk.
fn push_chunks(
    stream: &mut Stream<sodiumoxide::crypto::secretstream::Push>,
    plaintext: &[u8],
    chunksize: usize,
) -> Vec<u8> {
    let mut out = Vec::new();
    let mut start = 0;
    loop {
        let end = std::cmp::min(start + chunksize, plaintext.len());
        let last = end - start < chunksize;
        let tag = if last { Tag::Final } else { Tag::Message };
        out.extend_from_slice(&stream.push(&plaintext[start..end], None, tag).unwrap());
        start = end;
        if last {
            break;
        }
    }
    out
}

/// Decrypts a version 4 file using libsodium directly, so a test can prove that what Cloaker
/// writes is readable by the reference implementation (and therefore by Cloaker.js and by
/// Cloaker 4.0), not merely by Cloaker itself.
pub fn reference_decrypt_v4(path: &Path, password: &str) -> Result<Vec<u8>, String> {
    sodiumoxide::init().unwrap();
    let data = read(path).map_err(|e| e.to_string())?;
    let preamble = V4_SIGNATURE.len() + argon2id13::SALTBYTES + HEADERBYTES;
    if data.len() < preamble {
        return Err("file is too short".to_string());
    }
    if data[..4] != V4_SIGNATURE {
        return Err("not a version 4 cloaker file".to_string());
    }

    let mut salt = [0u8; argon2id13::SALTBYTES];
    salt.copy_from_slice(&data[4..4 + argon2id13::SALTBYTES]);
    let mut header = [0u8; HEADERBYTES];
    header.copy_from_slice(&data[4 + argon2id13::SALTBYTES..preamble]);

    let mut key_bytes = [0u8; KEYBYTES];
    argon2id13::derive_key(
        &mut key_bytes,
        password.as_bytes(),
        &argon2id13::Salt(salt),
        argon2id13::OPSLIMIT_INTERACTIVE,
        argon2id13::MEMLIMIT_INTERACTIVE,
    )
    .map_err(|_| "deriving key failed".to_string())?;
    let mut stream = Stream::init_pull(&Header(header), &Key(key_bytes))
        .map_err(|_| "init_pull failed".to_string())?;

    let mut plaintext = Vec::new();
    let mut rest = &data[preamble..];
    while stream.is_not_finalized() {
        if rest.is_empty() {
            return Err("file is truncated".to_string());
        }
        let take = std::cmp::min(V4_CHUNKSIZE + ABYTES, rest.len());
        let (chunk, tail) = rest.split_at(take);
        let (decrypted, _tag) = stream
            .pull(chunk, None)
            .map_err(|_| "incorrect password".to_string())?;
        plaintext.extend_from_slice(&decrypted);
        rest = tail;
    }
    Ok(plaintext)
}
