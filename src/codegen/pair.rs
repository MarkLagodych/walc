use super::*;

pub fn new(first: Expr, second: Expr) -> Expr {
    abs(["P"], apply(var("P"), [first, second]))
}

pub fn get_first(pair: Expr) -> Expr {
    apply(pair, [var("0")])
}

pub fn get_second(pair: Expr) -> Expr {
    apply(pair, [var("1")])
}
