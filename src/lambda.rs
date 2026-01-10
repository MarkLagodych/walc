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

pub fn call_recursive(function: Lambda, argument: Lambda) -> Lambda {
    call(call(function.clone(), function), argument)
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

pub fn pair(first: Lambda, second: Lambda) -> Lambda {
    fun("P", call(call(var("P"), first), second))
}

pub fn none() -> Lambda {
    pair(var("0"), unreachable())
}

pub fn some(value: Lambda) -> Lambda {
    pair(var("1"), value)
}

pub fn left(value: Lambda) -> Lambda {
    pair(var("0"), value)
}

pub fn right(value: Lambda) -> Lambda {
    pair(var("1"), value)
}

pub fn cond(condition: Lambda, then_branch: Lambda, else_branch: Lambda) -> Lambda {
    call(call(condition, else_branch), then_branch)
}

pub fn i32_const(mut value: u32) -> Lambda {
    let mut result = var("N");
    for _ in 0..32 {
        let bit = value & 1;
        value >>= 1;
        result = call(result, if bit == 1 { var("1") } else { var("0") });
    }

    fun("N", result)
}

pub fn define_prelude(root: Lambda) -> Lambda {
    let _0 = fun("x0", fun("x1", var("x0")));
    let _1 = fun("x0", fun("x1", var("x1")));

    let _end = none();

    let _out = fun(
        "out_byte",
        fun("next", some(left(pair(var("out_byte"), var("next"))))),
    );

    let _in = fun("in_func", some(right(var("in_func"))));

    let root = define(root, "In", _in);
    let root = define(root, "Out", _out);
    let root = define(root, "End", _end);
    let root = define(root, "1", _1);
    let root = define(root, "0", _0);
    root
}
