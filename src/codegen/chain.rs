use super::*;

pub fn empty() -> Expr {
    unreachable()
}

pub fn node(head: Expr, tail: Expr) -> Expr {
    // TODO Should unsafe list also use a predefined cons(tructor) like the safe list?
    pair::new(head, tail)
}

pub fn from(items: impl DoubleEndedIterator<Item = Expr>) -> Expr {
    let mut result = empty();
    for item in items.rev() {
        result = node(item, result);
    }
    result
}

pub fn from_bytes(store: &mut number::ConstantStore, bytes: &[u8]) -> Expr {
    from(bytes.iter().map(|b| store.byte_const(*b)))
}

pub fn get_head(list: Expr) -> Expr {
    pair::get_first(list)
}

pub fn get_tail(list: Expr) -> Expr {
    pair::get_second(list)
}
