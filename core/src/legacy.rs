use crate::os_interface::Ui;
use crate::{maybe_fill_buffer, percentage, CoreError};
use dryoc::constants::{
    CRYPTO_SECRETSTREAM_XCHACHA20POLY1305_ABYTES as ABYTES,
    CRYPTO_SECRETSTREAM_XCHACHA20POLY1305_HEADERBYTES as HEADERBYTES,
    CRYPTO_SECRETSTREAM_XCHACHA20POLY1305_KEYBYTES as KEYBYTES,
};
use dryoc::dryocstream::{DryocStream, Header, Key, Pull, Tag};
use std::error;
use std::io::prelude::*;
use zeroize::Zeroize;

const CHUNKSIZE: usize = 4096;
pub const SIGNATURE: [u8; 4] = [0xC1, 0x0A, 0x4B, 0xED];

// Cloaker 1.x to 3.x derived the key with libsodium's crypto_pwhash_scryptsalsa208sha256 at the
// "interactive" limits (opslimit 524288, memlimit 16777216). libsodium turns those limits into
// scrypt parameters internally; for these particular constants it picks N = 2^14, r = 8, p = 1.
// The compatibility tests check this against libsodium itself, so don't change them by hand.
const SALTBYTES: usize = 32; // crypto_pwhash_scryptsalsa208sha256_SALTBYTES
const LOG_N: u8 = 14;
const R: u32 = 8;
const P: u32 = 1;

pub fn decrypt<I: Read, O: Write>(
    input: &mut I,
    output: &mut O,
    password: &str,
    ui: &dyn Ui,
    filesize: Option<usize>,
    first_four: Option<[u8; 4]>,
) -> Result<(), Box<dyn error::Error>> {
    // make sure file is at least [prefix +] salt + header. cloaker 1.0 files have no prefix,
    // in which case the first four bytes have already been read and are part of the salt.
    if let Some(size) = filesize {
        let prefix_len = if first_four.is_some() {
            0
        } else {
            SIGNATURE.len()
        };
        if size < SALTBYTES + HEADERBYTES + prefix_len {
            return Err(CoreError::new("File not big enough to have been encrypted").into());
        }
    }
    let mut total_bytes_read = 0;

    let mut salt = [0u8; SALTBYTES];
    match first_four {
        Some(four) => {
            // if signature was not present, and we're treating this as a cloaker 1.0 file because of the
            // .cloaker extension or because -d was used from CLI, then use those bytes for the salt.
            salt[..4].copy_from_slice(&four);
            input.read_exact(&mut salt[4..])?;
        }
        None => input.read_exact(&mut salt)?,
    };

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

fn derive_key(password: &str, salt: &[u8; SALTBYTES]) -> Result<Key, CoreError> {
    let params =
        scrypt::Params::new(LOG_N, R, P).map_err(|_| CoreError::new("Deriving key failed"))?;
    let mut key_bytes = [0u8; KEYBYTES];
    scrypt::scrypt(password.as_bytes(), salt, &params, &mut key_bytes)
        .map_err(|_| CoreError::new("Deriving key failed"))?;
    let key = Key::from(key_bytes);
    key_bytes.zeroize();
    Ok(key)
}
