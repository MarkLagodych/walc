use super::*;

pub fn first(value: Expr) -> Expr {
    pair::new(var("0"), value)
}

pub fn second(value: Expr) -> Expr {
    pair::new(var("1"), value)
}

pub fn is_second(either: Expr) -> Expr {
    pair::get_first(either)
}

pub fn unwrap(either: Expr) -> Expr {
    pair::get_second(either)
}
