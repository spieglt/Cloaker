mod legacy;
mod os_interface;
pub use os_interface::*;

use std::{error, fmt};
use std::io::prelude::*;
use sodiumoxide::crypto::pwhash::argon2id13;
use sodiumoxide::crypto::secretstream::
    {Stream, Tag, ABYTES, HEADERBYTES, KEYBYTES};
use sodiumoxide::crypto::secretstream::xchacha20poly1305::{Header, Key};

const CHUNKSIZE: usize = 1024 * 512;
const SIGNATURE: [u8; 4] = [0xC1, 0x0A, 0x6B, 0xED];

#[derive(Debug)]
pub struct CoreError {
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

pub fn encrypt<I: Read, O: Write>(input: &mut I, output: &mut O, password: &str, ui: &Box<dyn Ui>, filesize: Option<usize>)
    -> Result<(), Box<dyn error::Error>> {

    let mut buffer = vec![0; CHUNKSIZE];
    let mut total_bytes_read = 0;
    
    // write file signature
    output.write(&SIGNATURE)?;

    let salt = argon2id13::gen_salt();
    output.write(&salt.0)?;

    let mut key = [0u8; KEYBYTES];
    argon2id13::derive_key(&mut key, password.as_bytes(), &salt,
        argon2id13::OPSLIMIT_INTERACTIVE,
        argon2id13::MEMLIMIT_INTERACTIVE).unwrap();
    let key = Key(key);
    let (mut stream, header) = Stream::init_push(&key)
        .map_err(|_| CoreError::new("init_push failed"))?;
    output.write(&header.0)?;
    let mut eof = false;
    while !eof {
        let res = maybe_fill_buffer(input, &mut buffer)?;
        eof = res.0;
        let bytes_read = res.1;
        total_bytes_read += bytes_read;
        let tag = if eof { Tag::Final } else { Tag::Message };
        if let Some(size) = filesize {
            let percentage = (((total_bytes_read as f32) / (size as f32)) * 100.) as i32;
            ui.output(percentage);
        }
        output.write(
            &stream.push(&buffer[..bytes_read], None, tag)
                .map_err(|_| CoreError::new("Encrypting file failed"))?
        )?;
    }

    Ok(())
}

pub fn decrypt<I: Read, O: Write>(input: &mut I, output: &mut O, password: &str, ui: &Box<dyn Ui>, filesize: Option<usize>)
    -> Result<(), Box<dyn error::Error>> {

    // make sure file is at least prefix + salt + header
    if let Some(size) = filesize {
        if !(size >= argon2id13::SALTBYTES + HEADERBYTES + SIGNATURE.len()) {
            return Err(CoreError::new("File not big enough to have been encrypted"))?;
        }
    }
    let mut total_bytes_read = 0;

    let mut salt = [0u8; argon2id13::SALTBYTES];
    input.read_exact(&mut salt)?;
    let salt = argon2id13::Salt(salt);

    let mut header = [0u8; HEADERBYTES];
    input.read_exact(&mut header)?;
    let header = Header(header);

    let mut key = [0u8; KEYBYTES];
    argon2id13::derive_key(&mut key, password.as_bytes(), &salt,
        argon2id13::OPSLIMIT_INTERACTIVE,
        argon2id13::MEMLIMIT_INTERACTIVE)
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
