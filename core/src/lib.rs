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
    let mut buf = [0; CHUNKSIZE];
    let mut bytes_left = in_file.metadata()?.len();
    
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

    loop {
        match (*in_file).read(&mut buf) {
            Ok(num_read) if num_read > 0 => {
                bytes_left -= num_read as u64;
                let tag = match bytes_left {
                    0 => Tag::Final,
                    _ => Tag::Message,
                };
                out_file.write(
                    &stream.push(&buf[..num_read], None, tag)
                        .map_err(|_| CoreError::new("Encrypting file failed"))?
                )?;
                continue
            },
            Err(e) => Err(e)?,
            _ => break, // reached EOF
        }
    }

    Ok(())
}

pub fn decrypt(in_file: &mut File, out_file: &mut File, password: &str)
    -> Result<(), Box<dyn error::Error>> {

    // make sure file is at least prefix + salt + header
    if !(in_file.metadata()?.len() > (pwhash::SALTBYTES + HEADERBYTES + SIGNATURE.len()) as u64) {
        return Err(CoreError::new("File not big enough to have been encrypted"))?;
    }

    let mut salt = [0u8; pwhash::SALTBYTES];
    let mut signature = [0u8; 4];

    in_file.read_exact(&mut signature)?;
    if signature == SIGNATURE { // if the signature is present, read into all of salt
        in_file.read_exact(&mut salt)?;
    } else { // or take the bytes from signature and read the rest from file
        &mut salt[..4].copy_from_slice(&signature);
        in_file.read_exact(&mut salt[4..])?;
    }
    let salt = pwhash::Salt(salt);

    let mut header = [0u8; HEADERBYTES];
    in_file.read_exact(&mut header)?;
    let header = Header(header);

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
        match in_file.read(&mut buffer) {
            Ok(num_read) if num_read > 0 => {
                let (decrypted, _tag) = stream.pull(&buffer[..num_read], None)
                    .map_err(|_| CoreError::new("Incorrect password"))?;
                out_file.write(&decrypted)?;
                continue
            },
            Err(e) => return Err(Box::new(e)),
            _ => return Err(CoreError::new("Decryption error"))?, // reached EOF
        }
    }
    Ok(())
}
