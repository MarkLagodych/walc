use super::*;

const BITNESS: u8 = 32;

pub fn new(initial_item: Expr) -> Expr {
    tree::new(BITNESS, initial_item)
}

pub fn index(memory: Expr, address: Expr) -> Expr {
    tree::index(memory, address)
}

pub fn insert(memory: Expr, address: Expr, value: Expr) -> Expr {
    tree::insert(BITNESS, memory, address, value)
}
