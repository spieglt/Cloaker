mod legacy;
mod os_interface;
pub use os_interface::*;

use std::{error, fmt};
use std::fs::File;
use std::io::prelude::*;
use sodiumoxide::crypto::pwhash::argon2id13;
use sodiumoxide::crypto::secretstream::
    {Stream, Tag, ABYTES, HEADERBYTES, KEYBYTES};
use sodiumoxide::crypto::secretstream::xchacha20poly1305::{Header, Key};

const CHUNKSIZE: usize = 1024 * 512;
const SIGNATURE: [u8; 4] = [0xC1, 0x0A, 0x6B, 0xED];

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


pub fn encrypt(in_file: &mut File, out_file: &mut File, password: &str) 
    -> Result<(), Box<dyn error::Error>> {

    let mut buffer = vec![0; CHUNKSIZE];
    
    // write file signature
    out_file.write(&SIGNATURE)?;

    let salt = argon2id13::gen_salt();
    out_file.write(&salt.0)?;

    let mut key = [0u8; KEYBYTES];
    argon2id13::derive_key(&mut key, password.as_bytes(), &salt,
        argon2id13::OPSLIMIT_INTERACTIVE,
        argon2id13::MEMLIMIT_INTERACTIVE).unwrap();
    let key = Key(key);
    let (mut stream, header) = Stream::init_push(&key)
        .map_err(|_| CoreError::new("init_push failed"))?;
    out_file.write(&header.0)?;
    let mut eof = false;
    while !eof {
        let res = maybe_fill_buffer(in_file, &mut buffer)?;
        eof = res.0;
        let bytes_read = res.1;
        let tag = if eof { Tag::Final } else { Tag::Message };
        out_file.write(
            &stream.push(&buffer[..bytes_read], None, tag)
                .map_err(|_| CoreError::new("Encrypting file failed"))?
        )?;
    }

    Ok(())
}

pub fn decrypt(in_file: &mut File, out_file: &mut File, password: &str)
    -> Result<(), Box<dyn error::Error>> {

    // TODO
    // let mut bytes_left = in_file.metadata()?.len() as usize;
    // // make sure file is at least prefix + salt + header
    // if !(bytes_left > argon2id13::SALTBYTES + HEADERBYTES + SIGNATURE.len()) {
    //     return Err(CoreError::new("File not big enough to have been encrypted"))?;
    // }

    let mut salt = [0u8; argon2id13::SALTBYTES];
    let mut signature = [0u8; 4];

    in_file.read_exact(&mut signature)?;
    if signature == SIGNATURE { // if the signature is present, read into all of salt
        in_file.read_exact(&mut salt)?;
    } else { // or take the bytes from signature and read the rest from file
        &mut salt[..4].copy_from_slice(&signature);
        in_file.read_exact(&mut salt[4..])?;
    }
    let salt = argon2id13::Salt(salt);

    let mut header = [0u8; HEADERBYTES];
    in_file.read_exact(&mut header)?;
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
        let (_eof, bytes_read) = maybe_fill_buffer(in_file, &mut buffer)?;
        if bytes_read != 0 {
            let (decrypted, _tag) = stream.pull(&buffer[..bytes_read], None)
                .map_err(|_| CoreError::new("Incorrect password"))?;
            out_file.write(&decrypted)?;
        }
    }
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
