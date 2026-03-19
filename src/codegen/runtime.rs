pub mod instructions;
pub mod math;

use super::*;

/// Generates all runtime utilities (instructions & maths) needed for the program.
///
/// The definitions are internally created on demand, meaning that only those instructions and
/// math operations that are actually used in the program will be generated,
/// which helps reduce code size without requiring a complex optimizer.
///
/// The definitions obey the dependency order as a result of the on-demand generation.
/// E.g. if you call `math::sub()` that requires `math::add()` internally, the addition operation
/// will be generated prior to substraction.
/// In code it looks like this:
/// ```
/// fn sub(rt: &mut RuntimeGenerator, a: Expr, b: Expr) -> Expr {
///     if !rt.has("SUB") {
///         let definition = abs(["a", "b"], {
///             let a = var("a");
///             let b = negate(rt, var("b"));
///             add(rt, a, b) // This defines ADD prior to SUB
///         });
///
///         rt.def("SUB", definition);
///     }
///
///     apply("SUB", [a, b])
/// }
///
/// fn add(rt: &mut RuntimeGenerator, a: Expr, b: Expr) -> Expr {
///     if !rt.has("ADD") {
///         ...
///         rt.def("ADD", ...);
///     }
///     apply("ADD", [a, b])
/// }
/// ```
#[derive(Default)]
pub struct RuntimeGenerator {
    pub num: number::NumberGenerator,

    defs: Vec<(String, Expr)>,
    already_defined: std::collections::HashSet<String>,
}

impl RuntimeGenerator {
    fn has(&self, definition_name: &str) -> bool {
        self.already_defined.contains(definition_name)
    }

    /// Before defining a new utility (or a set of utilities), first check if it has already
    /// been defined with [`Self::has`].
    fn def(&mut self, name: impl ToString, value: Expr) {
        self.already_defined.insert(name.to_string());
        self.defs.push((name.to_string(), value));
    }

    /// Same as [`Self::def`] but for recursive definitions.
    ///
    /// Before defining a new utility (or a set of utilities), first check if it has already
    /// been defined with [`Self::has`].
    fn def_rec(&mut self, name: impl ToString, value: Expr) {
        let value = abs([name.to_string()], value);
        self.already_defined.insert(name.to_string());
        self.defs.push((name.to_string(), value));
    }

    /// Generates variable definitions into the given `let..in` expression builder.
    pub fn generate(self, b: &mut LetExprBuilder) {
        self.num.generate(b);

        for (name, value) in self.defs.into_iter() {
            b.def(name, value);
        }
    }
}
