#![macro_use]

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
        left: Box<Lambda>,
        right: Box<Lambda>,
    },
}

impl std::fmt::Display for Lambda {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lambda::Variable { name } => write!(f, "{}", name),
            Lambda::Abstraction { variable, body } => {
                write!(f, "[{} {}]", variable, body)
            }
            Lambda::Application { left, right } => {
                write!(f, "({} {})", left, right)
            }
        }
    }
}

pub fn var(name: &str) -> Lambda {
    Lambda::Variable {
        name: name.to_string(),
    }
}

fn fun(variable: &str, body: Lambda) -> Lambda {
    Lambda::Abstraction {
        variable: variable.to_string(),
        body: Box::new(body),
    }
}

macro_rules! fun {
    ($var1:expr, $var2:expr, $($rest:expr),+ ) => {
        fun($var1, fun!($var2, $($rest),+))
    };
    ($var:expr, $body:expr) => {
        fun($var, $body)
    };
}

fn call(function: Lambda, argument: Lambda) -> Lambda {
    Lambda::Application {
        left: Box::new(function),
        right: Box::new(argument),
    }
}

macro_rules! call {
    ($func:expr, $arg:expr, $($rest:expr),+) => {
        call!(call($func, $arg), $($rest),+)
    };
    ($func:expr, $arg:expr) => {
        call($func, $arg)
    };
}

/// Applies the function to itself so that it can be called recursively
pub fn rec(function: Lambda) -> Lambda {
    call!(function.clone(), function)
}

pub fn define(root: Lambda, variable: &str, value: Lambda) -> Lambda {
    call!(fun!(variable, root), value)
}

pub fn define_recursive(root: Lambda, function_name: &str, body: Lambda) -> Lambda {
    call!(fun!(function_name, root), fun!(function_name, body))
}

pub fn unreachable() -> Lambda {
    if cfg!(feature = "unbound-unreachable") {
        var("__UNREACHABLE__")
    } else {
        fun!("_", var("_"))
    }
}

pub fn bit(b: bool) -> Lambda {
    if b { var("1") } else { var("0") }
}

pub fn cond(condition: Lambda, then_branch: Lambda, else_branch: Lambda) -> Lambda {
    call!(condition, else_branch, then_branch)
}

pub mod pair {
    use super::*;

    pub fn new(first: Lambda, second: Lambda) -> Lambda {
        fun!("P", call!(var("P"), first, second))
    }

    pub fn get_left(pair: Lambda) -> Lambda {
        call!(pair, var("0"))
    }

    pub fn get_right(pair: Lambda) -> Lambda {
        call!(pair, var("1"))
    }
}

pub mod either {
    use super::*;

    pub fn left(value: Lambda) -> Lambda {
        pair::new(bit(false), value)
    }

    pub fn right(value: Lambda) -> Lambda {
        pair::new(bit(true), value)
    }

    pub fn is_right(either: Lambda) -> Lambda {
        pair::get_left(either)
    }

    pub fn unwrap(either: Lambda) -> Lambda {
        pair::get_right(either)
    }
}

pub mod optional {
    use super::*;

    pub fn none() -> Lambda {
        either::left(unreachable())
    }

    pub fn some(value: Lambda) -> Lambda {
        either::right(value)
    }

    pub fn is_some(optional: Lambda) -> Lambda {
        either::is_right(optional)
    }

    pub fn unwrap(optional: Lambda) -> Lambda {
        either::unwrap(optional)
    }
}

pub mod number {
    use super::*;

    fn bytes_to_hex_string(le_bytes: &[u8]) -> String {
        String::from("0x")
            + &le_bytes
                .iter()
                .rev()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join("")
    }

    fn number_const(le_bytes: &[u8]) -> Lambda {
        let hex_string = bytes_to_hex_string(le_bytes);

        let mut result = var(&hex_string);
        for byte in le_bytes {
            let mut byte = *byte;
            for _ in 0..8 {
                result = call!(result, bit(byte & 1 == 1));
                byte >>= 1;
            }
        }

        fun(&hex_string, result)
    }

    pub fn u8_const(n: u8) -> Lambda {
        number_const(&n.to_le_bytes())
    }

    pub fn u32_const(n: u32) -> Lambda {
        number_const(&n.to_le_bytes())
    }

    pub fn u64_const(n: u64) -> Lambda {
        number_const(&n.to_le_bytes())
    }

    pub fn f32_const(n: f32) -> Lambda {
        number_const(&n.to_le_bytes())
    }

    pub fn f64_const(n: f64) -> Lambda {
        number_const(&n.to_le_bytes())
    }

    pub fn to_bit_list_be32(number: Lambda) -> Lambda {
        call!(var("ToBitsBE32"), number)
    }
}

pub mod dyn_list {
    use super::*;

    pub fn empty() -> Lambda {
        optional::none()
    }

    pub fn node(head: Lambda, tail: Lambda) -> Lambda {
        optional::some(pair::new(head, tail))
    }

    pub fn is_not_empty(list: Lambda) -> Lambda {
        optional::is_some(list)
    }

    pub fn get_head(list: Lambda) -> Lambda {
        pair::get_left(optional::unwrap(list))
    }

    pub fn get_tail(list: Lambda) -> Lambda {
        pair::get_right(optional::unwrap(list))
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

    pub fn get_head(list: Lambda) -> Lambda {
        pair::get_left(list)
    }

    pub fn get_tail(list: Lambda) -> Lambda {
        pair::get_right(list)
    }
}

pub mod array_tree {
    use super::*;

    pub fn default() -> Lambda {
        var("Array32")
    }

    pub fn node(left: Lambda, right: Lambda) -> Lambda {
        pair::new(left, right)
    }

    pub fn get_left(array: Lambda) -> Lambda {
        pair::get_left(array)
    }

    pub fn get_right(array: Lambda) -> Lambda {
        pair::get_right(array)
    }

    pub fn index(array: Lambda, index: Lambda) -> Lambda {
        call!(index, array)
    }

    pub fn insert(array: Lambda, index: Lambda, value: Lambda) -> Lambda {
        call!(
            var("ArrayInsert32"),
            array,
            number::to_bit_list_be32(index),
            value
        )
    }
}

pub mod walc_io {
    use super::*;

    pub fn end() -> Lambda {
        var("End")
    }

    pub fn output(root: Lambda, out_byte: Lambda) -> Lambda {
        call!(var("Out"), out_byte, root)
    }

    pub fn input(root_input_handler: Lambda) -> Lambda {
        call!(var("In"), root_input_handler)
    }
}

pub fn define_prelude(mut root: Lambda) -> Lambda {
    root = define_array_utils(root);
    root = define_walc_io(root);
    root = define_number_utils(root);
    root = define_bits(root);
    root
}

fn define_bits(mut root: Lambda) -> Lambda {
    root = define(root, "0", fun!("x0", "x1", var("x0")));
    root = define(root, "1", fun!("x0", "x1", var("x1")));
    root
}

fn define_number_utils(mut root: Lambda) -> Lambda {
    root = define_to_bits_be32(root);

    root
}

fn define_to_bits_be32(mut root: Lambda) -> Lambda {
    let mut expr = list::empty();

    for i in 0..32 {
        let bit_name = format!("b{}", i);
        expr = list::node(var(&bit_name), expr);
    }

    for i in (0..32).rev() {
        let bit_name = format!("b{}", i);
        expr = fun(&bit_name, expr);
    }

    root = define(root, "ToBitsBE32", expr);

    root
}

fn define_walc_io(mut root: Lambda) -> Lambda {
    root = define(
        root,
        "In",
        fun!(
            "input_handler",
            optional::some(either::right(var("input_handler")))
        ),
    );

    root = define(
        root,
        "Out",
        fun!(
            "out_byte",
            "next",
            optional::some(either::left(pair::new(var("out_byte"), var("next"))))
        ),
    );

    root = define(root, "End", optional::none());

    root
}

fn define_array_utils(mut root: Lambda) -> Lambda {
    root = define_array_templates(root);
    root = define_array_insert(root);

    root
}

fn define_array_templates(mut root: Lambda) -> Lambda {
    // The number indicates the depth of the tree, i.e. the number of bits in the index

    for i in (1..=32).rev() {
        root = define(
            root,
            &format!("Array{}", i),
            array_tree::node(
                var(&format!("Array{}", i - 1)),
                var(&format!("Array{}", i - 1)),
            ),
        );
    }

    root = define(root, "Array0", number::u8_const(b'a'));

    root
}

fn define_array_insert(mut root: Lambda) -> Lambda {
    for i in (1..=32).rev() {
        root = define(
            root,
            &format!("ArrayInsert{}", i),
            fun!(
                "array",
                "index",
                "value",
                cond(
                    list::get_head(var("index")),
                    array_tree::node(
                        array_tree::get_left(var("array")),
                        call!(
                            var(&format!("ArrayInsert{}", i - 1)),
                            array_tree::get_right(var("array")),
                            list::get_tail(var("index")),
                            var("value")
                        )
                    ),
                    array_tree::node(
                        call!(
                            var(&format!("ArrayInsert{}", i - 1)),
                            array_tree::get_left(var("array")),
                            list::get_tail(var("index")),
                            var("value")
                        ),
                        array_tree::get_right(var("array")),
                    ),
                )
            ),
        );
    }

    root = define(
        root,
        "ArrayInsert0",
        fun!("array", "index", "value", var("value")),
    );

    root
}
