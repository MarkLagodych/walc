/// WALC code generator
pub mod chain;
pub mod either;
pub mod list;
pub mod memory;
pub mod number;
pub mod op;
pub mod optional;
pub mod pair;
pub mod table;
pub mod tree;
pub mod walc_io;

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

    pub fn build(self, body: Expr) -> Expr {
        let mut result = body;
        for (var, value) in self.defs.into_iter().rev() {
            result = def(var, value, result);
        }
        result
    }

    pub fn define_prelude(&mut self) {
        self.def("0", abs(["x0", "x1"], var("x0")));
        self.def("1", abs(["x0", "x1"], var("x1")));

        walc_io::define_prelude(self);
        list::define_prelude(self);
        number::define_prelude(self);
        tree::define_prelude(self);
    }
}
