pub mod instruction;

pub mod code;

pub mod number;

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

/// `branch0` is selected if `condition` is 0, `branch1` is selected if `condition` is 1.
pub fn select(condition: Bit, branch0: Expr, branch1: Expr) -> Expr {
    apply(condition, [branch0, branch1])
}

pub fn unreachable() -> Expr {
    if cfg!(feature = "unbound-unreachable") {
        static mut NEXT_ID: u32 = 0;

        // Generates a unique identifier of an unbound variable.
        // We don't care about thread safety here
        let next_id = unsafe {
            let id = NEXT_ID;
            NEXT_ID += 1;
            id
        };

        var(format!("UNREACHABLE_{}", next_id))
    } else {
        var("_")
    }
}

pub type Bit = Expr;

pub fn bit(b: bool) -> Bit {
    if b { var("1") } else { var("0") }
}

/// Constructs a simple `let {var} = {value} in {body}` expression.
pub fn let_in(var: impl ToString, value: Expr, body: Expr) -> Expr {
    apply(abs([var], body), [value])
}

/// Applies the function to itself, which allows the function to call itself recursively.
pub fn rec(func: Expr) -> Expr {
    apply(func.clone(), [func])
}

/// Constructs a `let .. in` expression.
///
/// `let` expressions allow variable shadowing, which enables imperative-style code like this:
/// ```text
/// let x = make_x in
/// let x = (do_something_with_x x) in
/// let x = (do_something_else_with_x x) in
/// body_using_x
/// ```
#[derive(Default)]
pub struct LetExprBuilder {
    defs: Vec<(String, Expr)>,
}

impl LetExprBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Defines a variable.
    /// This is the same as adding a new `let {name} = {value} in` line  into a let expression.
    pub fn def(&mut self, name: impl ToString, value: Expr) {
        self.defs.push((name.to_string(), value));
    }

    /// Defines a recursive function, i.e. a function takes itself as the first argument and
    /// therefore can call itself.
    ///
    /// To call the function, use [`rec`].
    pub fn def_rec(&mut self, name: impl ToString, value: Expr) {
        self.defs.push((name.to_string(), abs([name], value)));
    }

    /// Combines all the definitions with the given body expression to form a complete
    /// `let .. in` expression:
    /// ```text
    /// let var1 = value1 in
    /// let var2 = value2 in
    /// ...
    /// let varN = valueN in
    /// body
    /// ```
    pub fn build_in(self, body: Expr) -> Expr {
        let mut result = body;
        for (var, value) in self.defs.into_iter().rev() {
            result = let_in(var, value, result);
        }
        result
    }
}

/// Provides definitions required for all core types and functions.
pub fn generate_core_definitions(b: &mut LetExprBuilder) {
    b.def("_", abs(["_"], var("_")));

    b.def("0", abs(["x0", "x1"], var("x0")));
    b.def("1", abs(["x0", "x1"], var("x1")));

    pair::generate_defs(b);
    list::generate_defs(b);
    number::generate_defs(b);
    tree::generate_defs(b);
}

pub mod io_command {
    use super::*;

    pub type IoCommand = Expr;

    pub fn exit() -> IoCommand {
        optional::none()
    }

    pub fn output(out_byte: number::Byte, next: Expr) -> IoCommand {
        optional::some(either::first(pair::new(out_byte, next)))
    }

    pub fn input(input_func: Expr) -> IoCommand {
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

    pub fn select(pair: Pair, selector: Bit) -> Expr {
        apply(pair, [selector])
    }

    pub fn generate_defs(b: &mut LetExprBuilder) {
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

    pub fn is_second(either: Either) -> Expr {
        pair::get_first(either)
    }

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
