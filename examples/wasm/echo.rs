mod walc;
use walc::*;

fn main() {
    while let Some(byte) = read_byte() {
        print_byte(byte);
    }
}
