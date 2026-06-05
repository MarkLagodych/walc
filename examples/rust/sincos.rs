#![no_std]
#![no_main]
mod walc;
use walc::*;

use cordic::sin_cos;
use fixed::types::I20F12 as Fixed;

pub fn main() {
    print_string("Enter angle in radians: ");

    let input = read_line();

    let angle = match Fixed::from_str(input.trim()) {
        Ok(val) => val,
        Err(_) => {
            print_string("Invalid input. Please enter a valid fixed-point number.\n");
            return;
        }
    };

    let (sin, cos) = sin_cos(angle);
    print_string("sin: ");
    print_string(sin.to_string().as_str());
    print_string("\ncos: ");
    print_string(cos.to_string().as_str());
    print_string("\n");
}
