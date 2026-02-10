#![no_std]
#![no_main]
mod walc;
use walc::*;

pub fn main() {
    while let Some(byte) = read_byte() {
        print_byte(byte);
    }
}
