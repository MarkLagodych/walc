//! WALC code generator

pub mod function;
pub mod number;
pub mod program;

mod lists;
pub use lists::*;

mod trees;
pub use trees::*;

use anyhow::{Result, anyhow};

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

                if matches!(**function, Expr::Abstraction { .. })
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

pub fn cond(condition: Expr, then_branch: Expr, else_branch: Expr) -> Expr {
    apply(condition, [else_branch, then_branch])
}

pub fn rec(func: Expr) -> Expr {
    apply(func.clone(), [func])
}

pub fn unreachable() -> Expr {
    if cfg!(feature = "unbound-unreachable") {
        var("__UNREACHABLE__")
    } else {
        abs(["_"], var("_"))
    }
}

pub fn bit(b: bool) -> Expr {
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

    pub fn def(&mut self, var: impl ToString, value: Expr) {
        self.defs.push((var.to_string(), value));
    }

    pub fn count(&self) -> usize {
        self.defs.len()
    }

    pub fn build(self, body: Expr) -> Expr {
        let mut result = body;
        for (var, value) in self.defs.into_iter().rev() {
            result = def(var, value, result);
        }
        result
    }

    /// Provides definitions required for all codegen features.
    pub fn prelude() -> Self {
        let mut me = Self::new();

        me.def("0", abs(["x0", "x1"], var("x0")));
        me.def("1", abs(["x0", "x1"], var("x1")));

        walc_io::define_prelude(&mut me);
        list::define_prelude(&mut me);
        number::define_prelude(&mut me);
        tree::define_prelude(&mut me);

        me
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
