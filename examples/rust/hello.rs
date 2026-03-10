#![no_std]
#![no_main]
mod walc;
use walc::*;

pub fn main() {
    print_string("Hello world!");
}
