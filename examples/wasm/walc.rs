#![allow(dead_code)]
#![macro_use]

#[link(wasm_import_module = "walc")]
unsafe extern "C" {
    fn output(c: u8);
    fn input() -> u32;
}

const INVALID_FLAG: u32 = 0x100;

pub fn print_byte(c: u8) {
    unsafe { output(c) }
}

pub fn read_byte() -> Option<u8> {
    let byte = unsafe { input() };
    if byte & INVALID_FLAG != 0 {
        None
    } else {
        Some(byte as u8)
    }
}

pub fn read_buffer(buffer: &mut [u8]) -> usize {
    let mut count = 0;
    for byte in buffer.iter_mut() {
        match read_byte() {
            Some(b) => {
                *byte = b;
                count += 1;
            }
            None => break,
        }
    }

    count
}

pub fn print_string(s: &str) {
    for &byte in s.as_bytes() {
        print_byte(byte);
    }
}

pub fn read_string() -> Result<String, std::string::FromUtf8Error> {
    let mut buffer = Vec::new();
    loop {
        match read_byte() {
            Some(byte) => buffer.push(byte),
            None => break,
        }
    }

    String::from_utf8(buffer)
}

pub fn read_line() -> Result<String, std::string::FromUtf8Error> {
    let mut buffer = Vec::new();
    loop {
        match read_byte() {
            Some(b'\n') => break,
            Some(byte) => buffer.push(byte),
            None => break,
        }
    }

    String::from_utf8(buffer)
}

macro_rules! print {
    ($($arg:tt)*) => ({
        let s = format!($($arg)*);
        print_string(&s);
    });
}

macro_rules! println {
    () => (print_string("\n"));
    ($($arg:tt)*) => ({
        print!($($arg)*);
        println!();
    });
}

macro_rules! eprint {
    ($($arg:tt)*) => ({
        println!($($arg)*);
    });
}

macro_rules! eprintln {
    () => {
        println!()
    };
    ($($arg:tt)*) => {{
        println!($($arg)*);
    }};
}

macro_rules! read {
    () => {{ read_string().expect("Cannot read UTF-8 string") }};
}

macro_rules! readln {
    () => {{ read_line().expect("Cannot read UTF-8 string") }};
}
