/// WALC code generator

#[derive(Debug, Clone)]
pub enum Expr {
    Variable {
        name: String,
    },
    Abstraction {
        variable: String,
        body: Box<Expr>,
    },
    Application {
        function: Box<Expr>,
        argument: Box<Expr>,
    },
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Variable { name } => {
                write!(f, "{}", name)
            }
            Expr::Abstraction { variable, body } => {
                write!(f, "[{}", variable)?;

                if let Expr::Variable { .. } = **body {
                    write!(f, " ")?;
                }

                write!(f, "{}]", body)
            }
            Expr::Application { function, argument } => {
                write!(f, "({}", function)?;

                if let Expr::Variable { .. } = **function
                    && let Expr::Variable { .. } = **argument
                {
                    write!(f, " ")?;
                }

                write!(f, "{})", argument)
            }
        }
    }
}

pub fn var(name: impl ToString) -> Expr {
    Expr::Variable {
        name: name.to_string(),
    }
}

pub fn abs<ToStr, Vars>(vars: Vars, body: Expr) -> Expr
where
    ToStr: ToString,
    Vars: IntoIterator<Item = ToStr>,
    Vars::IntoIter: DoubleEndedIterator<Item = ToStr>,
{
    let mut result = body;
    for var in vars.into_iter().rev() {
        result = Expr::Abstraction {
            variable: var.to_string(),
            body: Box::new(result),
        };
    }
    result
}

pub fn apply(func: Expr, args: impl IntoIterator<Item = Expr>) -> Expr {
    let mut result = func;
    for arg in args {
        result = Expr::Application {
            function: Box::new(result),
            argument: Box::new(arg),
        };
    }
    result
}

pub fn def(var: impl ToString, value: Expr, body: Expr) -> Expr {
    apply(abs([var], body), [value])
}

#[derive(Default)]
pub struct DefinitionBuilder {
    defs: Vec<(String, Expr)>,
}

impl DefinitionBuilder {
    pub fn new() -> Self {
        Self { defs: vec![] }
    }

    pub fn def(&mut self, var: impl ToString, value: Expr) {
        self.defs.push((var.to_string(), value));
    }

    pub fn build(self, body: Expr) -> Expr {
        let mut result = body;
        for (var, value) in self.defs.into_iter().rev() {
            result = def(var, value, result);
        }
        result
    }
}

pub fn cond(condition: Expr, then_branch: Expr, else_branch: Expr) -> Expr {
    apply(condition, [else_branch, then_branch])
}

pub fn unreachable() -> Expr {
    if cfg!(feature = "debug-unreachable") {
        var("__UNREACHABLE__")
    } else {
        abs(["_"], var("_"))
    }
}

pub fn rec(func: Expr) -> Expr {
    apply(func.clone(), [func])
}

pub fn bit(b: bool) -> Expr {
    if b { var("1") } else { var("0") }
}

fn bit_define_prelude(b: &mut DefinitionBuilder) {
    b.def("0", abs(["x0", "x1"], var("x0")));
    b.def("1", abs(["x0", "x1"], var("x1")));
}

pub mod pair {
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
}

pub mod either {
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
}

pub mod optional {
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
}

pub mod safe_list {
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

    pub fn from_bytes(store: &mut number::ConstantStore, bytes: &[u8]) -> Expr {
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
            // head, tail
            abs(["h", "t"], optional::some(pair::new(var("h"), var("t")))),
        );
    }
}

pub mod list {
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
}

pub mod walc_io {
    use super::*;

    pub fn end() -> Expr {
        var("End")
    }

    pub fn output(out_byte: Expr, next: Expr) -> Expr {
        apply(var("Out"), [out_byte, next])
    }

    pub fn input(root_input_handler: Expr) -> Expr {
        apply(var("In"), [root_input_handler])
    }

    pub fn define_prelude(b: &mut DefinitionBuilder) {
        b.def("End", optional::none());

        b.def(
            "Out",
            abs(
                ["out_byte", "next"],
                optional::some(either::first(pair::new(var("out_byte"), var("next")))),
            ),
        );

        b.def(
            "In",
            abs(
                ["input_handler"],
                optional::some(either::second(var("input_handler"))),
            ),
        );
    }
}

pub mod number {
    use super::*;

    use std::collections::HashSet;

    /// This struct accumulates all numeric constants throughout the WASM module in order to
    /// reduce the resulting code size.
    #[derive(Default)]
    pub struct ConstantStore {
        byte_set: HashSet<u8>,
        i32_set: HashSet<u32>,
        i64_set: HashSet<u64>,
    }

    impl ConstantStore {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn define_constants(&self, b: &mut DefinitionBuilder) {
            for &c in &self.byte_set {
                b.def(format!("{:02x}", c), number_const(&c.to_be_bytes()));
            }
            for &c in &self.i32_set {
                b.def(format!("{:08x}", c), number_const(&c.to_be_bytes()));
            }
            for &c in &self.i64_set {
                b.def(format!("{:016x}", c), number_const(&c.to_be_bytes()));
            }
        }

        pub fn byte_const(&mut self, n: u8) -> Expr {
            self.byte_set.insert(n);
            var(format!("{:02x}", n))
        }

        pub fn i32_const(&mut self, n: u32) -> Expr {
            self.i32_set.insert(n);
            var(format!("{:08x}", n))
        }

        pub fn i64_const(&mut self, n: u64) -> Expr {
            self.i64_set.insert(n);
            var(format!("{:016x}", n))
        }
    }

    fn number_const(be_bytes: &[u8]) -> Expr {
        fn ith_bit(byte: u8, i: u8) -> Expr {
            bit((byte >> i) & 1u8 != 0)
        }

        abs(
            ["N"],
            apply(
                var("N"),
                be_bytes
                    .iter()
                    .flat_map(|byte| (0..8).rev().map(|i| ith_bit(*byte, i))),
            ),
        )
    }

    pub fn to_bit_list_be32(number: Expr) -> Expr {
        // I debugged this for two weeks :X
        // Yes, you really call a number with a function, not the other way around.
        apply(number, [var("ToBitsBE32")])
    }

    pub fn define_prelude(b: &mut DefinitionBuilder) {
        b.def(
            "ToBitsBE32",
            abs(
                (0..32).rev().map(|i| i.to_string()),
                list::from((0..32).rev().map(|i| var(i.to_string()))),
            ),
        );
    }
}

pub mod tree {
    use super::*;

    pub fn new(default_item: Expr) -> Expr {
        apply(var("TreeOf"), [default_item])
    }

    pub fn node(left: Expr, right: Expr) -> Expr {
        pair::new(left, right)
    }

    pub fn get_left(array: Expr) -> Expr {
        pair::get_first(array)
    }

    pub fn get_right(array: Expr) -> Expr {
        pair::get_second(array)
    }

    pub fn index(array: Expr, index: Expr) -> Expr {
        apply(index, [array])
    }

    pub fn insert(array: Expr, index: Expr, value: Expr) -> Expr {
        apply(var("TreeInsert"), [array, index, value])
    }

    pub fn define_prelude(b: &mut DefinitionBuilder) {
        // Indexable by 0 bits (i.e. not indexable)
        b.def("T0", abs(["x"], var("x")));
        // Every node is indexable by i bits
        for i in 1..=32 {
            let node_name = format!("T{}", i);
            let item_name = format!("T{}", i - 1);
            let item = apply(var(item_name), [var("x")]);
            b.def(node_name, abs(["x"], tree::node(item.clone(), item)));
        }

        b.def("TreeOf", var("T32"));

        b.def(
            "TIns_",
            abs(
                ["insert", "array", "index", "value"],
                cond(
                    list::get_head(var("index")),
                    tree::node(
                        tree::get_left(var("array")),
                        apply(
                            var("insert"),
                            [
                                tree::get_right(var("array")),
                                list::get_tail(var("index")),
                                var("value"),
                            ],
                        ),
                    ),
                    tree::node(
                        apply(
                            var("insert"),
                            [
                                tree::get_left(var("array")),
                                list::get_tail(var("index")),
                                var("value"),
                            ],
                        ),
                        tree::get_right(var("array")),
                    ),
                ),
            ),
        );

        b.def("TIns0", abs(["array", "index", "value"], var("value")));

        // Each insertion function consumes i bits of the index
        for i in 1..=32 {
            b.def(
                format!("TIns{}", i),
                apply(var("TIns_"), [var(format!("TIns{}", i - 1))]),
            );
        }

        b.def(
            "TreeInsert",
            abs(
                ["array", "index", "value"],
                apply(
                    var("TIns32"),
                    [
                        var("array"),
                        number::to_bit_list_be32(var("index")),
                        var("value"),
                    ],
                ),
            ),
        );
    }
}

pub fn define_prelude(b: &mut DefinitionBuilder) {
    bit_define_prelude(b);
    walc_io::define_prelude(b);
    safe_list::define_prelude(b);
    number::define_prelude(b);
    tree::define_prelude(b);
}
