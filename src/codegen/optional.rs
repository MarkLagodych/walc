use super::*;

pub fn none() -> Expr {
    either::first(unreachable())
}

pub fn some(value: Expr) -> Expr {
    either::second(value)
}

pub fn is_some(optional: Expr) -> Expr {
    either::is_second(optional)
}

pub fn unwrap(optional: Expr) -> Expr {
    either::unwrap(optional)
}
