use super::*;

const BITNESS: u8 = 16;

pub fn new() -> Expr {
    tree::new(BITNESS, unreachable())
}

pub fn from(items: impl IntoIterator<Item = Expr>) -> Expr {
    tree::from(BITNESS, items, unreachable())
}

pub fn index(table: Expr, address: Expr) -> Expr {
    tree::index(table, address)
}

pub fn insert(table: Expr, address: Expr, value: Expr) -> Expr {
    tree::insert(BITNESS, table, address, value)
}
