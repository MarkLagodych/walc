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

pub fn print_buffer(buffer: &[u8]) {
    for &byte in buffer {
        print_byte(byte);
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

pub fn read_all() -> Vec<u8> {
    let mut data = Vec::new();
    loop {
        let mut buf = [0u8; 1024];
        let len = read_buffer(&mut buf);
        if len == 0 {
            break;
        }

        data.extend_from_slice(&buf[..len]);
    }

    data
}

pub fn print_string(s: &str) {
    print_buffer(s.as_bytes());
}

pub fn read_string() -> Result<String, std::string::FromUtf8Error> {
    String::from_utf8(read_all())
}

pub fn read_line() -> Result<String, std::string::FromUtf8Error> {
    let mut line = Vec::new();
    loop {
        match read_byte() {
            Some(b'\n') => break,
            Some(byte) => line.push(byte),
            None => break,
        }
    }
    String::from_utf8(line)
}

macro_rules! print {
    ($($arg:tt)*) => ({
        let s = format!($($arg)*);
        print_string(&s);
    })
}

macro_rules! println {
    () => (print_string("\n"));
    ($($arg:tt)*) => ({
        print!($($arg)*);
        println!();
    })
}
