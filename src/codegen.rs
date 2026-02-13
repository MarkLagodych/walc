//! WALC code generator

pub mod function;
pub mod instruction;
pub mod number;
pub mod program;

mod lists;
pub use lists::*;

mod trees;
pub use trees::*;

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

                if matches!(**body, Expr::Variable { .. }) {
                    write!(f, " ")?;
                }

                write!(f, "{}]", body)
            }
            Expr::Application { function, argument } => {
                write!(f, "({}", function)?;

                if matches!(**function, Expr::Variable { .. })
                    && matches!(**argument, Expr::Variable { .. })
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

/// `branch0` is selected if `condition` is 0, `branch1` is selected if `condition` is 1.
pub fn select(condition: Bit, branch0: Expr, branch1: Expr) -> Expr {
    apply(condition, [branch0, branch1])
}

pub fn unreachable() -> Expr {
    if cfg!(feature = "unbound-unreachable") {
        var("__UNREACHABLE__")
    } else {
        abs(["_"], var("_"))
    }
}

pub type Bit = Expr;

pub fn bit(b: bool) -> Bit {
    if b { var("1") } else { var("0") }
}

#[derive(Default)]
pub struct DefinitionBuilder {
    defs: Vec<(String, Expr)>,
}

impl DefinitionBuilder {
    pub fn new() -> Self {
        Self { defs: vec![] }
    }

    pub fn def(&mut self, name: impl ToString, value: Expr) {
        self.defs.push((name.to_string(), value));
    }

    pub fn def_rec(&mut self, name: impl ToString, value: Expr) {
        self.defs.push((
            name.to_string(),
            apply(var("Y"), [abs([name.to_string()], value)]),
        ));
    }

    pub fn build(self, body: Expr) -> Expr {
        let mut result = body;
        for (var, value) in self.defs.into_iter().rev() {
            result = def(var, value, result);
        }
        result
    }

    /// Provides definitions required for all basic codegen features.
    pub fn prelude() -> Self {
        let mut me = Self::new();

        me.def("0", abs(["x0", "x1"], var("x0")));
        me.def("1", abs(["x0", "x1"], var("x1")));

        // Y combinator
        me.def(
            "Y",
            abs(
                ["f"],
                apply(
                    abs(["x"], apply(var("f"), [apply(var("x"), [var("x")])])),
                    [abs(["x"], apply(var("f"), [apply(var("x"), [var("x")])]))],
                ),
            ),
        );

        pair::define_prelude(&mut me);
        list::define_prelude(&mut me);
        number::define_prelude(&mut me);
        tree::define_prelude(&mut me);

        me
    }
}

pub mod io_command {
    use super::*;

    pub fn exit() -> Expr {
        optional::none()
    }

    pub fn output(out_byte: number::Byte, next: Expr) -> Expr {
        optional::some(either::first(pair::new(out_byte, next)))
    }

    pub fn input(input_func: Expr) -> Expr {
        optional::some(either::second(input_func))
    }
}

pub mod pair {
    use super::*;

    pub type Pair = Expr;

    pub fn new(first: Expr, second: Expr) -> Pair {
        apply(var("P"), [first, second])
    }

    pub fn get_first(pair: Pair) -> Expr {
        apply(pair, [var("0")])
    }

    pub fn get_second(pair: Pair) -> Expr {
        apply(pair, [var("1")])
    }

    pub fn define_prelude(b: &mut DefinitionBuilder) {
        b.def(
            "P",
            abs(
                ["first", "second"],
                abs(["g"], apply(var("g"), [var("first"), var("second")])),
            ),
        );
    }
}

pub mod either {
    use super::*;

    pub type Either = Expr;

    pub fn first(value: Expr) -> Either {
        pair::new(var("0"), value)
    }

    pub fn second(value: Expr) -> Either {
        pair::new(var("1"), value)
    }

    #[allow(unused)]
    pub fn is_second(either: Either) -> Expr {
        pair::get_first(either)
    }

    #[allow(unused)]
    pub fn unwrap(either: Either) -> Expr {
        pair::get_second(either)
    }
}

pub mod optional {
    use super::*;

    pub type Optional = Expr;

    pub fn none() -> Optional {
        either::first(unreachable())
    }

    pub fn some(value: Expr) -> Optional {
        either::second(value)
    }

    pub fn is_some(optional: Optional) -> Expr {
        either::is_second(optional)
    }

    pub fn unwrap(optional: Optional) -> Expr {
        either::unwrap(optional)
    }
}
