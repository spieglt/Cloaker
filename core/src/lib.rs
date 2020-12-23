mod os_interface;
pub use os_interface::*;

use std::{error, fmt};
use std::fs::File;
use std::io::prelude::*;
use sodiumoxide::crypto::pwhash;
use sodiumoxide::crypto::secretstream::
    {Stream, Tag, ABYTES, HEADERBYTES, KEYBYTES};
use sodiumoxide::crypto::secretstream::xchacha20poly1305::{Header, Key};

const CHUNKSIZE: usize = 4096;
const SIGNATURE: [u8; 4] = [0xC1, 0x0A, 0x4B, 0xED];

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

    // change to vec to allocate on heap and have larger chunksize
    // was overflowing stack when set to 1_000_000
    let mut buffer = [0; CHUNKSIZE];
    let mut bytes_left = in_file.metadata()?.len() as usize;
    
    // write file signature
    out_file.write(&SIGNATURE)?;

    let salt = pwhash::gen_salt();
    out_file.write(&salt.0)?;

    let mut key = [0u8; KEYBYTES];
    pwhash::derive_key(&mut key, password.as_bytes(), &salt,
        pwhash::OPSLIMIT_INTERACTIVE,
        pwhash::MEMLIMIT_INTERACTIVE).unwrap();
    let key = Key(key);
    let (mut stream, header) = Stream::init_push(&key)
        .map_err(|_| CoreError::new("init_push failed"))?;
    out_file.write(&header.0)?;

    while bytes_left > 0 {
        if bytes_left > buffer.len() {
            in_file.read_exact(&mut buffer)?;
            bytes_left -= buffer.len();
            out_file.write(
                &stream.push(&buffer, None, Tag::Message)
                    .map_err(|_| CoreError::new("Encrypting file failed"))?
            )?;
        } else {
            let mut remainder = vec![];
            in_file.read_to_end(&mut remainder)?;
            bytes_left -= remainder.len();
            out_file.write(
                &stream.push(&remainder, None, Tag::Final)
                    .map_err(|_| CoreError::new("Encrypting file failed"))?
            )?;
        }
    }

    Ok(())
}

pub fn decrypt(in_file: &mut File, out_file: &mut File, password: &str)
    -> Result<(), Box<dyn error::Error>> {

    let mut bytes_left = in_file.metadata()?.len() as usize;
    // make sure file is at least prefix + salt + header
    if !(bytes_left > pwhash::SALTBYTES + HEADERBYTES + SIGNATURE.len()) {
        return Err(CoreError::new("File not big enough to have been encrypted"))?;
    }

    let mut salt = [0u8; pwhash::SALTBYTES];
    let mut signature = [0u8; 4];

    in_file.read_exact(&mut signature)?;
    bytes_left -= signature.len();
    if signature == SIGNATURE { // if the signature is present, read into all of salt
        in_file.read_exact(&mut salt)?;
        bytes_left -= pwhash::SALTBYTES;
    } else { // or take the bytes from signature and read the rest from file
        &mut salt[..4].copy_from_slice(&signature);
        in_file.read_exact(&mut salt[4..])?;
        bytes_left -= pwhash::SALTBYTES - 4;
    }
    let salt = pwhash::Salt(salt);

    let mut header = [0u8; HEADERBYTES];
    in_file.read_exact(&mut header)?;
    let header = Header(header);
    bytes_left -= HEADERBYTES;

    let mut key = [0u8; KEYBYTES];
    pwhash::derive_key(&mut key, password.as_bytes(), &salt,
        pwhash::OPSLIMIT_INTERACTIVE,
        pwhash::MEMLIMIT_INTERACTIVE)
        .map_err(|_| CoreError::new("Deriving key failed"))?;
    let key = Key(key);

    let mut buffer = [0u8; CHUNKSIZE + ABYTES];
    let mut stream = Stream::init_pull(&header, &key)
        .map_err(|_| CoreError::new("init_pull failed"))?;

    while stream.is_not_finalized() {
        if bytes_left > buffer.len() {
            in_file.read_exact(&mut buffer)?;
            bytes_left -= buffer.len();
            let (decrypted, _tag) = stream.pull(&buffer, None)
                .map_err(|_| CoreError::new("Incorrect password"))?;
            out_file.write(&decrypted)?;
        } else {
            let mut remainder = vec![];
            in_file.read_to_end(&mut remainder)?;
            bytes_left -= remainder.len();
            let (decrypted, _tag) = stream.pull(&remainder, None)
                .map_err(|_| CoreError::new("Incorrect password"))?;
            out_file.write(&decrypted)?;
        }
    }
    Ok(())
}
