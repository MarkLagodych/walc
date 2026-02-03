#[derive(Debug, Clone)]
pub enum Lambda {
    Variable {
        name: String,
    },
    Abstraction {
        variable: String,
        body: Box<Lambda>,
    },
    Application {
        function: Box<Lambda>,
        argument: Box<Lambda>,
    },
}

impl std::fmt::Display for Lambda {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lambda::Variable { name } => {
                write!(f, "{}", name)
            }
            Lambda::Abstraction { variable, body } => {
                write!(f, "[{} {}]", variable, body)
            }
            Lambda::Application { function, argument } => {
                write!(f, "({} {})", function, argument)
            }
        }
    }
}

pub fn var(name: impl ToString) -> Lambda {
    Lambda::Variable {
        name: name.to_string(),
    }
}

pub fn abs<ToStr, Vars>(vars: Vars, body: Lambda) -> Lambda
where
    ToStr: ToString,
    Vars: IntoIterator<Item = ToStr>,
    Vars::IntoIter: DoubleEndedIterator<Item = ToStr>,
{
    let mut result = body;
    for var in vars.into_iter().rev() {
        result = Lambda::Abstraction {
            variable: var.to_string(),
            body: Box::new(result),
        };
    }
    result
}

pub fn apply(func: Lambda, args: impl IntoIterator<Item = Lambda>) -> Lambda {
    let mut result = func;
    for arg in args {
        result = Lambda::Application {
            function: Box::new(result),
            argument: Box::new(arg),
        };
    }
    result
}

pub fn def(var: impl ToString, value: Lambda, body: Lambda) -> Lambda {
    apply(abs([var], body), [value])
}

pub struct DefinitionBuilder {
    defs: Vec<(String, Lambda)>,
}

impl DefinitionBuilder {
    pub fn new() -> Self {
        Self { defs: vec![] }
    }

    pub fn def(&mut self, var: impl ToString, value: Lambda) {
        self.defs.push((var.to_string(), value));
    }

    pub fn build(self, body: Lambda) -> Lambda {
        let mut result = body;
        for (var, value) in self.defs.into_iter().rev() {
            result = def(var, value, result);
        }
        result
    }
}

pub fn cond(condition: Lambda, then_branch: Lambda, else_branch: Lambda) -> Lambda {
    apply(condition, [else_branch, then_branch])
}

pub fn unreachable() -> Lambda {
    if cfg!(feature = "debug-unreachable") {
        var("__UNREACHABLE__")
    } else {
        abs(["U"], var("U"))
    }
}

pub fn rec(func: Lambda) -> Lambda {
    apply(func.clone(), [func])
}

pub fn bit(b: bool) -> Lambda {
    if b { var("1") } else { var("0") }
}

pub mod pair {
    use super::*;

    pub fn new(first: Lambda, second: Lambda) -> Lambda {
        abs(["P"], apply(var("P"), [first, second]))
    }

    pub fn get_first(pair: Lambda) -> Lambda {
        apply(pair, [var("0")])
    }

    pub fn get_second(pair: Lambda) -> Lambda {
        apply(pair, [var("1")])
    }
}

pub mod either {
    use super::*;

    pub fn first(value: Lambda) -> Lambda {
        pair::new(var("0"), value)
    }

    pub fn second(value: Lambda) -> Lambda {
        pair::new(var("1"), value)
    }

    pub fn is_second(either: Lambda) -> Lambda {
        pair::get_first(either)
    }

    pub fn unwrap(either: Lambda) -> Lambda {
        pair::get_second(either)
    }
}

pub mod optional {
    use super::*;

    pub fn none() -> Lambda {
        either::first(unreachable())
    }

    pub fn some(value: Lambda) -> Lambda {
        either::second(value)
    }

    pub fn is_some(optional: Lambda) -> Lambda {
        either::is_second(optional)
    }

    pub fn unwrap(optional: Lambda) -> Lambda {
        either::unwrap(optional)
    }
}

pub mod safe_list {
    use super::*;

    pub fn empty() -> Lambda {
        optional::none()
    }

    pub fn node(head: Lambda, tail: Lambda) -> Lambda {
        optional::some(pair::new(head, tail))
    }

    pub fn from(items: impl DoubleEndedIterator<Item = Lambda>) -> Lambda {
        let mut result = empty();
        for item in items.rev() {
            result = node(item, result);
        }
        result
    }

    pub fn is_not_empty(list: Lambda) -> Lambda {
        optional::is_some(list)
    }

    pub fn get_head(list: Lambda) -> Lambda {
        pair::get_first(optional::unwrap(list))
    }

    pub fn get_tail(list: Lambda) -> Lambda {
        pair::get_second(optional::unwrap(list))
    }
}

pub mod list {
    use super::*;

    pub fn empty() -> Lambda {
        unreachable()
    }

    pub fn node(head: Lambda, tail: Lambda) -> Lambda {
        pair::new(head, tail)
    }

    pub fn from(items: impl DoubleEndedIterator<Item = Lambda>) -> Lambda {
        let mut result = empty();
        for item in items.rev() {
            result = node(item, result);
        }
        result
    }

    pub fn get_head(list: Lambda) -> Lambda {
        pair::get_first(list)
    }

    pub fn get_tail(list: Lambda) -> Lambda {
        pair::get_second(list)
    }
}

pub mod tree_list {
    use super::*;

    pub fn default() -> Lambda {
        var("Arr")
    }

    pub fn node(left: Lambda, right: Lambda) -> Lambda {
        pair::new(left, right)
    }

    pub fn get_left(array: Lambda) -> Lambda {
        pair::get_first(array)
    }

    pub fn get_right(array: Lambda) -> Lambda {
        pair::get_second(array)
    }

    pub fn index(array: Lambda, index: Lambda) -> Lambda {
        apply(index, [array])
    }

    pub fn insert(array: Lambda, index: Lambda, value: Lambda) -> Lambda {
        apply(var("ArrInsert"), [array, index, value])
    }
}

pub mod walc_io {
    use super::*;

    pub fn end() -> Lambda {
        var("End")
    }

    pub fn output(out_byte: Lambda, next: Lambda) -> Lambda {
        apply(var("Out"), [out_byte, next])
    }

    pub fn input(root_input_handler: Lambda) -> Lambda {
        apply(var("In"), [root_input_handler])
    }
}

pub mod number {
    use super::*;

    fn bytes_to_hex_string(be_bytes: &[u8]) -> String {
        String::from("0x")
            + &be_bytes
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<String>>()
                .join("")
    }

    fn number_const(be_bytes: &[u8]) -> Lambda {
        let var_name = bytes_to_hex_string(be_bytes);

        let ith_bit = |byte: u8, i: usize| -> Lambda { bit((byte >> i) & 1 != 0) };

        abs(
            [var_name.clone()],
            apply(
                var(var_name),
                be_bytes
                    .iter()
                    .flat_map(|byte| (0..8).rev().map(|i| ith_bit(*byte, i))),
            ),
        )
    }

    pub fn u8_const(n: u8) -> Lambda {
        number_const(&n.to_be_bytes())
    }

    pub fn u32_const(n: u32) -> Lambda {
        number_const(&n.to_be_bytes())
    }

    pub fn u64_const(n: u64) -> Lambda {
        number_const(&n.to_be_bytes())
    }

    pub fn f32_const(n: f32) -> Lambda {
        number_const(&n.to_be_bytes())
    }

    pub fn f64_const(n: f64) -> Lambda {
        number_const(&n.to_be_bytes())
    }

    pub fn to_bit_list_be32(number: Lambda) -> Lambda {
        // I debugged this for two weeks :X
        // Yes, you really call a number with a function, not the other way around.
        apply(number, [var("ToBitsBE32")])
    }
}

pub fn prelude() -> DefinitionBuilder {
    let mut b = DefinitionBuilder::new();

    b.def("0", abs(["x0", "x1"], var("x0")));
    b.def("1", abs(["x0", "x1"], var("x1")));
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

    b.def("End", optional::none());

    b.def(
        "ToBitsBE32",
        abs(
            (0..32).rev().map(|i| i.to_string()),
            list::from((0..32).rev().map(|i| var(i.to_string()))),
        ),
    );

    // Default item, indexable by 0 bits (i.e. not indexable)
    b.def("Arr0", number::u8_const(0));

    // Every node is indexable by i bits
    for i in 1..=32 {
        let node_name = format!("Arr{}", i);
        let item_name = format!("Arr{}", i - 1);
        b.def(node_name, tree_list::node(var(&item_name), var(&item_name)));
    }

    b.def("Arr", var("Arr32"));

    b.def(
        "ArrInsert_",
        abs(
            ["insert", "array", "index", "value"],
            cond(
                list::get_head(var("index")),
                tree_list::node(
                    tree_list::get_left(var("array")),
                    apply(
                        var("insert"),
                        [
                            tree_list::get_right(var("array")),
                            list::get_tail(var("index")),
                            var("value"),
                        ],
                    ),
                ),
                tree_list::node(
                    apply(
                        var("insert"),
                        [
                            tree_list::get_left(var("array")),
                            list::get_tail(var("index")),
                            var("value"),
                        ],
                    ),
                    tree_list::get_right(var("array")),
                ),
            ),
        ),
    );

    b.def("ArrInsert0", abs(["array", "index", "value"], var("value")));

    // Each insertion function consumes i bits of the index
    for i in 1..=32 {
        b.def(
            format!("ArrInsert{}", i),
            apply(var("ArrInsert_"), [var(format!("ArrInsert{}", i - 1))]),
        );
    }

    b.def(
        "ArrInsert",
        abs(
            ["array", "index", "value"],
            apply(
                var("ArrInsert32"),
                [
                    var("array"),
                    number::to_bit_list_be32(var("index")),
                    var("value"),
                ],
            ),
        ),
    );

    b
}
