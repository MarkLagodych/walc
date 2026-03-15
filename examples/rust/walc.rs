#![allow(dead_code)]
#![allow(unused)]
#![macro_use]
#![no_std]

pub extern crate alloc;
pub use alloc::{
    borrow::ToOwned,
    boxed::Box,
    format,
    string::{String, ToString},
    vec::Vec,
};

mod walc {
    #[link(wasm_import_module = "walc")]
    unsafe extern "C" {
        pub fn exit() -> !;
        pub fn output(c: u8);
        pub fn input() -> u32;
    }
}

pub fn print_byte(c: u8) {
    unsafe { walc::output(c) }
}

pub fn read_byte() -> Option<u8> {
    let result = unsafe { walc::input() };
    if result == 0xFFFFFFFF {
        None
    } else {
        Some(result as u8)
    }
}

pub fn exit() -> ! {
    unsafe { walc::exit() }
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
    let mut buffer = Vec::new();

    while let Some(byte) = read_byte() {
        buffer.push(byte);
    }

    buffer
}

pub fn print_string(s: &str) {
    for &byte in s.as_bytes() {
        print_byte(byte);
    }
}

pub fn read_string() -> String {
    String::from_utf8(read_all()).expect("Invalid UTF-8 input")
}

pub fn read_line() -> String {
    let mut buffer = Vec::new();

    while let Some(byte) = read_byte() {
        if byte == b'\n' {
            break;
        }
        buffer.push(byte);
    }

    String::from_utf8(buffer).expect("Invalid UTF-8 input")
}

pub fn print_u32(mut n: u32) {
    let mut buffer = [0u8; 10];
    let mut i = 0;

    loop {
        buffer[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
        if n == 0 {
            break;
        }
    }

    for j in (0..i).rev() {
        print_byte(buffer[j]);
    }
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

#[unsafe(export_name = "main")]
fn main() {
    crate::main()
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print_string("\n!!!FATAL ERROR!!!\n");

    if let Some(location) = info.location() {
        print_string("Location: ");
        print_string(location.file());
        print_string(":");
        print_u32(location.line());
        print_string("\n");
    }

    if let Some(msg) = info.message().as_str() {
        print_string("Message: ");
        print_string(msg);
        print_string("\n");
    }

    exit()
}

#[global_allocator]
static ALLOCATOR: talc::TalckWasm = unsafe { talc::TalckWasm::new_global() };
