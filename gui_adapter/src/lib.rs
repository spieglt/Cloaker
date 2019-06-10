use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr::null_mut;

use cloaker::{Config, Mode, main_routine};

#[no_mangle]
pub extern fn makeConfig(mode: u8, password: *mut c_char, filename: *mut c_char) -> *mut Config {
    let m = match mode {
        0 => Mode::Encrypt,
        1 => Mode::Decrypt,
        _ => panic!("received invalid mode enum from c++"),
    };
    let p = match c_to_rust_string(password) {
        Ok(s) => s,
        Err(_) => return null_mut(),
    };
    let f = match c_to_rust_string(filename) {
        Ok(s) => s,
        Err(_) => return null_mut(),
    };
    Box::into_raw(Box::new(Config::new(m, p, f)))
}

#[no_mangle]
pub extern fn start(ptr: *mut Config) -> *mut c_char {
    let config = unsafe { &mut *ptr };
    let msg = match main_routine(config) {
        Ok(filename) => {
            match config.mode {
                Mode::Encrypt => format!("Success! Encrypted file {} is alongside the original.", filename),
                Mode::Decrypt => format!("Success! Decrypted file {} is alongside the original.", filename),
            }
        },
        Err(e) => format!("{}", e),
    };
    rust_to_c_string(msg)
}

#[no_mangle]
pub unsafe extern fn destroyConfig(ptr: *mut Config) {
    if ptr != null_mut() {
        drop(Box::from_raw(&mut *ptr));
    }
}

#[no_mangle]
pub unsafe extern fn destroyCString(ptr: *mut c_char) {
    if ptr != null_mut() {
        drop(CString::from_raw(ptr));
    }
}

fn rust_to_c_string(s: String) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

fn c_to_rust_string(ptr: *mut c_char) -> Result<String, String> {
    let c_str: &CStr = unsafe { CStr::from_ptr(ptr) };
    let res = c_str.to_str()
        .map_err(|_| "Could not convert C string to Rust string".to_string())?
        .to_string();
    Ok(res)
}
