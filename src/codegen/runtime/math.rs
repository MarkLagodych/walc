mod arithmetic;
mod comparisons;
mod conversions;
mod logical;

pub use arithmetic::*;
pub use comparisons::*;
pub use conversions::*;
pub use logical::*;

use super::*;

fn bit_not(a: Bit) -> Bit {
    select(a, bit(true), bit(false))
}

fn bit_and(a: Bit, b: Bit) -> Bit {
    select(a, bit(false), b)
}

fn bit_or(a: Bit, b: Bit) -> Bit {
    select(a, b, bit(true))
}

fn bit_xor(a: Bit, b: Bit) -> Bit {
    select(a, b.clone(), bit_not(b))
}

fn bit_equal(a: Bit, b: Bit) -> Bit {
    select(a, bit_not(b.clone()), b)
}
