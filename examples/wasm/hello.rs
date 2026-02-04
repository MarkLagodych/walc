#![no_std]
#![no_main]
mod walc;
use walc::*;

#[unsafe(no_mangle)]
fn main() {
    println!("Hello world!");
}
