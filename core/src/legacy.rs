use crate::os_interface::Ui;
use std::{error, fmt};
use std::io::prelude::*;
use sodiumoxide::crypto::pwhash;
use sodiumoxide::crypto::secretstream::
    {Stream, ABYTES, HEADERBYTES, KEYBYTES};
use sodiumoxide::crypto::secretstream::xchacha20poly1305::{Header, Key};

const CHUNKSIZE: usize = 4096;
pub const SIGNATURE: [u8; 4] = [0xC1, 0x0A, 0x4B, 0xED];

#[derive(Debug)]
struct CoreError {
    message: String,
}

impl CoreError {
    fn new(msg: &str) -> Self { CoreError{message: msg.to_string()} }
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl error::Error for CoreError {}

pub fn decrypt<I: Read, O: Write>(input: &mut I, output: &mut O, password: &str, ui: &Box<dyn Ui>, filesize: Option<usize>, first_four: Option<[u8; 4]>)
    -> Result<(), Box<dyn error::Error>> {

    // make sure file is at least prefix + salt + header
    if let Some(size) = filesize {
        if !(size >= pwhash::SALTBYTES + HEADERBYTES + SIGNATURE.len()) {
            return Err(CoreError::new("File not big enough to have been encrypted"))?;
        }
    }
    let mut total_bytes_read = 0;

    let mut salt = [0u8; pwhash::SALTBYTES];
    match first_four {
        Some(four) => {
            // if signature was not present, and we're treating this as a cloaker 1.0 file because of the
            // .cloaker extension or because -d was used from CLI, then use those bytes for the salt.
            &mut salt[..4].copy_from_slice(&four);
            input.read_exact(&mut salt[4..])?;
        },
        None => input.read_exact(&mut salt)?,
    };
    let salt = pwhash::Salt(salt);

    let mut header = [0u8; HEADERBYTES];
    input.read_exact(&mut header)?;
    let header = Header(header);

    let mut key = [0u8; KEYBYTES];
    pwhash::derive_key(&mut key, password.as_bytes(), &salt,
        pwhash::OPSLIMIT_INTERACTIVE,
        pwhash::MEMLIMIT_INTERACTIVE)
        .map_err(|_| CoreError::new("Deriving key failed"))?;
    let key = Key(key);

    let mut buffer = vec![0u8; CHUNKSIZE + ABYTES];
    let mut stream = Stream::init_pull(&header, &key)
        .map_err(|_| CoreError::new("init_pull failed"))?;
    while stream.is_not_finalized() {
        let (_eof, bytes_read) = maybe_fill_buffer(input, &mut buffer)?;
        total_bytes_read += bytes_read;
        let (decrypted, _tag) = stream.pull(&buffer[..bytes_read], None)
            .map_err(|_| CoreError::new("Incorrect password"))?;
        if let Some(size) = filesize {
            let percentage = (((total_bytes_read as f32) / (size as f32)) * 100.) as i32;
            ui.output(percentage);
        }
        output.write(&decrypted)?;
    }
    ui.output(100);
    Ok(())
}

// returns Ok(true, bytes_read) if EOF, and Ok(false, bytes_read) if buffer was filled without EOF
fn maybe_fill_buffer<T: Read>(reader: &mut T, buffer: &mut Vec<u8>) -> std::io::Result<(bool, usize)> {
    let mut bytes_read = 0;
    while bytes_read < buffer.len() {
        match reader.read(&mut buffer[bytes_read..]) {
            Ok(x) if x == 0 => return Ok((true, bytes_read)), // EOF
            Ok(x) => bytes_read += x,
            Err(e) => return Err(e),
        };
    }
    Ok((false, bytes_read))
}
