use super::*;

pub mod chain {
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

    pub fn from_bytes(store: &mut number::ConstantSet, bytes: &[u8]) -> Expr {
        from(bytes.iter().map(|b| store.byte_const(*b)))
    }

    pub fn get_head(list: Expr) -> Expr {
        pair::get_first(list)
    }

    pub fn get_tail(list: Expr) -> Expr {
        pair::get_second(list)
    }
}

pub mod list {
    use super::*;

    pub fn empty() -> Expr {
        var("Empty")
    }

    pub fn node(head: Expr, tail: Expr) -> Expr {
        apply(var("C"), [head, tail])
    }

    pub fn from(items: impl DoubleEndedIterator<Item = Expr>) -> Expr {
        let mut result = empty();
        for item in items.rev() {
            result = node(item, result);
        }
        result
    }

    pub fn from_bytes(store: &mut number::ConstantSet, bytes: &[u8]) -> Expr {
        from(bytes.iter().map(|b| store.byte_const(*b)))
    }

    pub fn is_not_empty(list: Expr) -> Expr {
        optional::is_some(list)
    }

    pub fn get_head(list: Expr) -> Expr {
        pair::get_first(optional::unwrap(list))
    }

    pub fn get_tail(list: Expr) -> Expr {
        pair::get_second(optional::unwrap(list))
    }

    pub fn define_prelude(b: &mut DefinitionBuilder) {
        b.def("Empty", optional::none());

        b.def(
            // Cons (list constructor)
            "C",
            abs(
                ["head", "tail"],
                optional::some(pair::new(var("head"), var("tail"))),
            ),
        );
    }
}
