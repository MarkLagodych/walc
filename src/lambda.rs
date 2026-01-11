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

pub fn fun(variable: &str, body: Lambda) -> Lambda {
    Lambda::Abstraction {
        variable: variable.to_string(),
        body: Box::new(body),
    }
}

pub fn call(function: Lambda, argument: Lambda) -> Lambda {
    Lambda::Application {
        left: Box::new(function),
        right: Box::new(argument),
    }
}

/// Applies the function to itself so that it can be called recursively
pub fn rec(function: Lambda) -> Lambda {
    call(function.clone(), function)
}

pub fn define(root: Lambda, variable: &str, value: Lambda) -> Lambda {
    call(fun(variable, root), value)
}

pub fn define_recursive(root: Lambda, variable: &str, value: Lambda) -> Lambda {
    call(fun(variable, root), fun(variable, value))
}

pub fn unreachable() -> Lambda {
    fun("_", var("_"))
}

pub fn bit(b: bool) -> Lambda {
    if b { var("1") } else { var("0") }
}

pub fn cond(condition: Lambda, then_branch: Lambda, else_branch: Lambda) -> Lambda {
    call(call(condition, else_branch), then_branch)
}

pub mod pair {
    use super::*;

    pub fn new(first: Lambda, second: Lambda) -> Lambda {
        fun("P", call(call(var("P"), first), second))
    }

    pub fn get_first(pair: Lambda) -> Lambda {
        call(pair, var("0"))
    }

    pub fn get_second(pair: Lambda) -> Lambda {
        call(pair, var("1"))
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
        pair::get_first(either)
    }

    pub fn unwrap(either: Lambda) -> Lambda {
        pair::get_second(either)
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
                result = call(result, bit(byte & 1 == 1));
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
        call(var("ToBitsBE32"), number)
    }
}

pub mod list {
    use super::*;

    pub fn empty() -> Lambda {
        optional::none()
    }

    pub fn cons(head: Lambda, tail: Lambda) -> Lambda {
        optional::some(pair::new(head, tail))
    }

    pub fn is_not_empty(list: Lambda) -> Lambda {
        optional::is_some(list)
    }

    pub fn head(list: Lambda) -> Lambda {
        pair::get_first(optional::unwrap(list))
    }

    pub fn tail(list: Lambda) -> Lambda {
        pair::get_second(optional::unwrap(list))
    }
}

pub mod tree {
    use super::*;

    pub fn new(left: Lambda, right: Lambda) -> Lambda {
        pair::new(left, right)
    }

    pub fn get_left(tree: Lambda) -> Lambda {
        pair::get_first(tree)
    }

    pub fn get_right(tree: Lambda) -> Lambda {
        pair::get_second(tree)
    }
}

pub mod array {
    use super::*;

    pub fn new() -> Lambda {
        var("Array32")
    }

    pub fn index(array: Lambda, index: Lambda) -> Lambda {
        call(index, array)
    }

    pub fn insert(array: Lambda, index: Lambda, value: Lambda) -> Lambda {
        call(call(var("ArrayInsert"), index), call(array, value))
    }
}

pub mod walc_command {
    use super::*;

    pub fn end() -> Lambda {
        var("End")
    }

    pub fn output(root: Lambda, out_byte: Lambda) -> Lambda {
        call(call(var("Out"), out_byte), root)
    }

    pub fn input(root_input_handler: Lambda) -> Lambda {
        call(var("In"), root_input_handler)
    }
}

pub fn define_prelude(mut root: Lambda) -> Lambda {
    root = define_array_utils(root);
    root = define_walc_commands(root);
    root = define_number_utils(root);
    root = define_bits(root);
    root
}

fn define_bits(mut root: Lambda) -> Lambda {
    root = define(root, "0", fun("x0", fun("x1", var("x0"))));
    root = define(root, "1", fun("x0", fun("x1", var("x1"))));
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
        expr = list::cons(var(&bit_name), expr);
    }

    for i in (0..32).rev() {
        let bit_name = format!("b{}", i);
        expr = fun(&bit_name, expr);
    }

    root = define(root, "ToBitsBE32", expr);

    root
}

fn define_walc_commands(mut root: Lambda) -> Lambda {
    let expr = optional::some(either::right(var("input_handler")));
    let expr = fun("input_handler", expr);
    root = define(root, "In", expr);

    let expr = optional::some(either::left(pair::new(var("out_byte"), var("next"))));
    let expr = fun("out_byte", fun("next", expr));
    root = define(root, "Out", expr);

    root = define(root, "End", optional::none());

    root
}

fn define_array_utils(mut root: Lambda) -> Lambda {
    root = define_array_templates(root);
    root = define_array_insert(root);

    root
}

fn define_array_templates(mut root: Lambda) -> Lambda {
    // Array numbers represent the number of bits used to address them.
    // Array1 is addressable by 1 bit (2 elements)
    // Array17 is addressable by 17 bits
    // Array32 is addressable by 32 bits

    for i in (2..32).rev() {
        let array_name = format!("Array{}", i);
        let smaller_array_name = format!("Array{}", i - 1);

        root = define(
            root,
            &array_name,
            tree::new(var(&smaller_array_name), var(&smaller_array_name)),
        );
    }

    root = define(
        root,
        "Array1",
        tree::new(var("ArrayDefaultItem_"), var("ArrayDefaultItem_")),
    );

    root = define(root, "ArrayDefaultItem_", number::u8_const(0));

    root
}

fn define_array_insert(mut root: Lambda) -> Lambda {
    let call_array_insert_rec =
        |array, index, value| call(call(call(rec(var("ArrayInsert_")), array), index), value);

    let expr = cond(
        list::is_not_empty(var("index")),
        cond(
            list::head(var("index")),
            call_array_insert_rec(
                tree::get_left(var("array")),
                list::tail(var("index")),
                var("value"),
            ),
            call_array_insert_rec(
                tree::get_right(var("array")),
                list::tail(var("index")),
                var("value"),
            ),
        ),
        var("value"),
    );

    let expr = fun("array", fun("index", fun("value", expr)));
    root = define_recursive(root, "ArrayInsert_", expr);

    let expr = call_array_insert_rec(
        var("array"),
        number::to_bit_list_be32(var("index")),
        var("value"),
    );
    let expr = fun("array", fun("index", fun("value", expr)));
    root = define(root, "ArrayInsert", expr);

    root
}
