#![no_std]
#![no_main]
mod walc;
use walc::*;

pub fn main() {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..100 {
        print!("{a} ");
        (a, b) = (b, a + b);
    }
}
