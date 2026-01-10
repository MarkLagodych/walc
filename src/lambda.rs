#![allow(clippy::let_and_return)]
#![allow(clippy::just_underscores_and_digits)]

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

pub fn funx(variables: &[&str], body: Lambda) -> Lambda {
    let mut result = body;
    for &variable in variables.iter().rev() {
        result = fun(variable, result);
    }
    result
}

pub fn callx(function: Lambda, arguments: impl Iterator<Item = Lambda>) -> Lambda {
    let mut result = function;
    for argument in arguments {
        result = call(result, argument);
    }
    result
}

pub fn call_recursive(function: Lambda, argument: Lambda) -> Lambda {
    callx(function.clone(), [function, argument].into_iter())
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
    callx(condition, [else_branch, then_branch].into_iter())
}

pub mod pair {
    use super::*;

    pub fn new(first: Lambda, second: Lambda) -> Lambda {
        fun("P", callx(var("P"), [first, second].into_iter()))
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

fn number_const(le_bytes: &[u8]) -> Lambda {
    let mut result = var("N");
    for byte in le_bytes {
        let mut byte = *byte;
        for _ in 0..8 {
            result = call(result, bit(byte & 1 == 1));
            byte >>= 1;
        }
    }

    fun("N", result)
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

pub fn define_prelude(root: Lambda) -> Lambda {
    let _0 = funx(&["x0", "x1"], var("x0"));
    let _1 = funx(&["x0", "x1"], var("x1"));

    let _end = optional::none();

    let _out = funx(
        &["out_byte", "next"],
        optional::some(either::left(pair::new(var("out_byte"), var("next")))),
    );

    let _in = fun(
        "input_handler",
        optional::some(either::right(var("input_handler"))),
    );

    let root = define(root, "In", _in);
    let root = define(root, "Out", _out);
    let root = define(root, "End", _end);
    let root = define(root, "1", _1);
    let root = define(root, "0", _0);
    root
}

pub mod walc {
    use super::*;

    pub fn end() -> Lambda {
        var("End")
    }

    pub fn output(root: Lambda, out_byte: Lambda) -> Lambda {
        callx(var("Out"), [out_byte, root].into_iter())
    }

    pub fn input(root_input_handler: Lambda) -> Lambda {
        call(var("In"), root_input_handler)
    }
}

pub fn compile_wasm(m: crate::wasm::Module) -> Lambda {
    let root = walc::end();
    let root = walc::output(root, u8_const(b'o'));
    let root = walc::output(root, u8_const(b'l'));
    let root = walc::output(root, u8_const(b'l'));
    let root = walc::output(root, u8_const(b'e'));
    let root = walc::output(root, u8_const(b'H'));

    let root = define_prelude(root);
    root
}
