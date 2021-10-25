mod legacy;
mod os_interface;
pub use os_interface::*;

use argon2::{
    password_hash::{
        rand_core::{OsRng, RngCore},
        PasswordHasher, SaltString
    },
    Argon2, Algorithm, ParamsBuilder, Version,
};
use crypto_secretstream::*;
use std::io::prelude::*;
use std::{error, fmt};

const CHUNKSIZE: usize = 1024 * 512;
const SIGNATURE: [u8; 4] = [0xC1, 0x0A, 0x6B, 0xED];
const SALTBYTES: usize = 16;
const KEYBYTES: usize = 32;
const ABYTES: usize = 17;

#[derive(Debug)]
pub struct CoreError {
    message: String,
}

impl CoreError {
    fn new(msg: &str) -> Self {
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
    ui: &Box<dyn Ui>,
    filesize: Option<usize>,
) -> Result<(), Box<dyn error::Error>> {
    let mut total_bytes_read = 0;

    // write file signature
    output.write_all(&SIGNATURE)?;

    let mut salt_bytes = [0; SALTBYTES];
    OsRng.fill_bytes(&mut salt_bytes);
    output.write_all(&salt_bytes)?;
    let salt = SaltString::b64_encode(&salt_bytes).map_err(|e| e.to_string())?;
    let key = get_key(password, salt)?;
    let (header, mut stream) = PushStream::init(&mut rand_core::OsRng, &key);
    output.write_all(header.as_ref())?;

    let mut eof = false;
    while !eof {
        let res = read_up_to(input, CHUNKSIZE)?;
        eof = res.0;
        let mut buffer = res.1;
        total_bytes_read += buffer.len();
        let tag = if eof { Tag::Final } else { Tag::Message };
        if let Some(size) = filesize {
            let percentage = (((total_bytes_read as f32) / (size as f32)) * 100.) as i32;
            ui.output(percentage);
        }
        stream.push(&mut buffer, &[], tag).map_err(|e| e.to_string())?;
        output.write_all(&buffer)?;
    }

    Ok(())
}

pub fn decrypt<I: Read, O: Write>(
    input: &mut I,
    output: &mut O,
    password: &str,
    ui: &Box<dyn Ui>,
    filesize: Option<usize>,
) -> Result<(), Box<dyn error::Error>> {
    // make sure file is at least prefix + salt + header
    if let Some(size) = filesize {
        if !(size >= SALTBYTES + Header::BYTES + SIGNATURE.len()) {
            return Err(CoreError::new("File not big enough to have been encrypted"))?;
        }
    }
    let mut total_bytes_read = 0;

    let mut salt = [0u8; SALTBYTES];
    input.read_exact(&mut salt)?;
    let salt = SaltString::b64_encode(&salt).map_err(|e| e.to_string())?;

    let mut header = [0u8; Header::BYTES];
    input.read_exact(&mut header)?;
    let header = Header::from(header);
    let key = get_key(password, salt)?;
    let mut stream = PullStream::init(header, &key);

    let mut tag = Tag::Message;
    while tag != Tag::Final {
        let (_eof, mut buffer) = read_up_to(input, CHUNKSIZE + ABYTES)?;
        total_bytes_read += buffer.len();
        tag = stream.pull(&mut buffer, &[]).map_err(|e| e.to_string())?;
        if let Some(size) = filesize {
            let percentage = (((total_bytes_read as f32) / (size as f32)) * 100.) as i32;
            ui.output(percentage);
        }
        output.write_all(&buffer)?;
    }
    ui.output(100);
    Ok(())
}

// returns Ok(true, buffer) if EOF, and Ok(false, buffer) if buffer was filled without EOF
fn read_up_to<R: Read>(
    reader: &mut R,
    limit: usize,
) -> std::io::Result<(bool, Vec<u8>)> {
    let mut bytes_read = 0;
    let mut buffer = vec![0u8; limit];
    while bytes_read < limit {
        match reader.read(&mut buffer[bytes_read..]) {
            Ok(x) if x == 0 => { // EOF
                buffer.truncate(bytes_read);
                return Ok((true, buffer))
            },
            Ok(x) => bytes_read += x,
            Err(e) => return Err(e),
        };
    }
    buffer.truncate(bytes_read);
    Ok((false, buffer))
}

fn get_key(password: &str, salt: SaltString) -> Result<Key, Box <dyn error::Error>> {
    let mut pb = ParamsBuilder::new();
    pb.m_cost(0x10000).map_err(|e| e.to_string())?;
    pb.t_cost(2).map_err(|e| e.to_string())?;
    let params = pb.params().map_err(|e| e.to_string())?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let key = argon2.hash_password(password.as_bytes(), &salt).map_err(|e| e.to_string())?;
    let key_hash = key.hash.ok_or_else(|| "\nno hash in key")?;
    let key_bytes = key_hash.as_bytes();
    let mut key_array = [0u8; KEYBYTES];
    for i in 0..key_array.len() {
        key_array[i] = key_bytes[i];
    }
    Ok(Key::from(key_array))
}
