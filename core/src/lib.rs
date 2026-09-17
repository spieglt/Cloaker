mod legacy;
mod os_interface;
mod paths;
pub use os_interface::*;
pub use paths::*;

use dryoc::constants::{
    CRYPTO_SECRETSTREAM_XCHACHA20POLY1305_ABYTES as ABYTES,
    CRYPTO_SECRETSTREAM_XCHACHA20POLY1305_HEADERBYTES as HEADERBYTES,
    CRYPTO_SECRETSTREAM_XCHACHA20POLY1305_KEYBYTES as KEYBYTES,
};
use dryoc::dryocstream::{DryocStream, Header, Key, Pull, Push, Tag};
use dryoc::rng::copy_randombytes;
use std::io::prelude::*;
use std::{error, fmt};
use zeroize::Zeroize;

const CHUNKSIZE: usize = 1024 * 512;
const SIGNATURE: [u8; 4] = [0xC1, 0x0A, 0x6B, 0xED];

// The key derivation parameters of the version 4 format, which are libsodium's crypto_pwhash
// defaults for argon2id13 at the "interactive" limits. They are not recorded in the file, so
// changing any of them makes every existing Cloaker file undecryptable. Cloaker.js has to match.
pub(crate) const SALTBYTES: usize = 16; // crypto_pwhash_argon2id_SALTBYTES
const OPSLIMIT_INTERACTIVE: u32 = 2; // crypto_pwhash_argon2id_OPSLIMIT_INTERACTIVE
const MEMLIMIT_INTERACTIVE_KIB: u32 = 65536; // crypto_pwhash_argon2id_MEMLIMIT_INTERACTIVE / 1024

#[derive(Debug)]
pub struct CoreError {
    message: String,
}

impl CoreError {
    pub(crate) fn new(msg: &str) -> Self {
        CoreError {
            message: msg.to_string(),
        }
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl error::Error for CoreError {}

pub fn encrypt<I: Read, O: Write>(
    input: &mut I,
    output: &mut O,
    password: &str,
    ui: &dyn Ui,
    filesize: Option<usize>,
) -> Result<(), Box<dyn error::Error>> {
    let mut buffer = vec![0; CHUNKSIZE];
    let mut total_bytes_read = 0;

    // write file signature
    output.write_all(&SIGNATURE)?;

    let mut salt = [0u8; SALTBYTES];
    copy_randombytes(&mut salt);
    output.write_all(&salt)?;

    let key = derive_key(password, &salt)?;
    let (mut stream, header): (DryocStream<Push>, Header) = DryocStream::init_push(&key);
    output.write_all(&header)?;
    let mut eof = false;
    while !eof {
        let (at_eof, bytes_read) = maybe_fill_buffer(input, &mut buffer)?;
        eof = at_eof;
        total_bytes_read += bytes_read;
        let tag = if eof { Tag::FINAL } else { Tag::MESSAGE };
        if let Some(size) = filesize {
            ui.output(percentage(total_bytes_read, size));
        }
        let chunk: &[u8] = &buffer[..bytes_read];
        output.write_all(
            &stream
                .push_to_vec(&chunk, None, tag)
                .map_err(|_| CoreError::new("Encrypting file failed"))?,
        )?;
    }
    output.flush()?;
    ui.output(100);

    Ok(())
}

pub fn decrypt<I: Read, O: Write>(
    input: &mut I,
    output: &mut O,
    password: &str,
    ui: &dyn Ui,
    filesize: Option<usize>,
) -> Result<(), Box<dyn error::Error>> {
    // make sure file is at least prefix + salt + header
    if let Some(size) = filesize {
        if size < SALTBYTES + HEADERBYTES + SIGNATURE.len() {
            return Err(CoreError::new("File not big enough to have been encrypted").into());
        }
    }
    let mut total_bytes_read = 0;

    let mut salt = [0u8; SALTBYTES];
    input.read_exact(&mut salt)?;

    let mut header_bytes = [0u8; HEADERBYTES];
    input.read_exact(&mut header_bytes)?;
    let header = Header::from(header_bytes);

    let key = derive_key(password, &salt)?;

    let mut buffer = vec![0u8; CHUNKSIZE + ABYTES];
    let mut stream: DryocStream<Pull> = DryocStream::init_pull(&key, &header);
    let mut chunks_read = 0;
    loop {
        let (_eof, bytes_read) = maybe_fill_buffer(input, &mut buffer)?;
        if bytes_read == 0 {
            // ran out of input before the stream said it was finished
            return Err(CoreError::new("File is truncated or corrupt").into());
        }
        total_bytes_read += bytes_read;
        let chunk: &[u8] = &buffer[..bytes_read];
        let (decrypted, tag) = stream.pull_to_vec(&chunk, None).map_err(|_| {
            if chunks_read == 0 {
                CoreError::new("Incorrect password")
            } else {
                // an earlier chunk decrypted, so the password is right and the file is damaged
                CoreError::new("File is truncated or corrupt")
            }
        })?;
        chunks_read += 1;
        if let Some(size) = filesize {
            ui.output(percentage(total_bytes_read, size));
        }
        output.write_all(&decrypted)?;
        if tag == Tag::FINAL {
            break;
        }
    }
    output.flush()?;
    ui.output(100);
    Ok(())
}

// argon2id v1.3, one lane, matching what libsodium's crypto_pwhash does with the constants above
fn derive_key(password: &str, salt: &[u8; SALTBYTES]) -> Result<Key, CoreError> {
    let params = argon2::Params::new(
        MEMLIMIT_INTERACTIVE_KIB,
        OPSLIMIT_INTERACTIVE,
        1,
        Some(KEYBYTES),
    )
    .map_err(|_| CoreError::new("Deriving key failed"))?;
    let argon2 = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let mut key_bytes = [0u8; KEYBYTES];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key_bytes)
        .map_err(|_| CoreError::new("Deriving key failed"))?;
    let key = Key::from(key_bytes);
    // Key zeroes itself when dropped, but this copy of the bytes would otherwise be left behind
    key_bytes.zeroize();
    Ok(key)
}

pub(crate) fn percentage(bytes_read: usize, size: usize) -> i32 {
    if size == 0 {
        return 100;
    }
    let fraction = (bytes_read as f64) / (size as f64);
    (fraction * 100.).min(100.) as i32
}

// returns Ok(true, bytes_read) if EOF, and Ok(false, bytes_read) if buffer was filled without EOF
pub(crate) fn maybe_fill_buffer<R: Read>(
    reader: &mut R,
    buffer: &mut [u8],
) -> std::io::Result<(bool, usize)> {
    let mut bytes_read = 0;
    while bytes_read < buffer.len() {
        match reader.read(&mut buffer[bytes_read..]) {
            Ok(0) => return Ok((true, bytes_read)), // EOF
            Ok(x) => bytes_read += x,
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
    }
    Ok((false, bytes_read))
}
