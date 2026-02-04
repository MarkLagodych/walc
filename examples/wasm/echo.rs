#![no_std]
#![no_main]
mod walc;
use walc::*;

#[unsafe(no_mangle)]
fn main() {
    while let Some(byte) = read_byte() {
        print_byte(byte);
    }
}
