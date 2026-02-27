mod instructions;
pub use instructions::*;

mod math;

use super::*;

use std::collections::HashSet as Set;

/// Generates all runtime utilities (instructions & maths) needed for the program.
///
/// The definitions are internally created on demand, with no name sorting,
/// but obeying the depdendency order.
#[derive(Default)]
pub struct UtilGenerator {
    pub num: number::NumberGenerator,

    defs: Vec<(String, Expr)>,
    already_defined: Set<String>,
}

impl UtilGenerator {
    fn has(&self, definition_name: &str) -> bool {
        self.already_defined.contains(definition_name)
    }

    /// Before defining a new utility, first check if it has already been defined with [`Self::has`]
    fn def(&mut self, name: impl ToString, value: Expr) {
        self.already_defined.insert(name.to_string());
        self.defs.push((name.to_string(), value));
    }

    /// Same as [`Self::def`] but for recursive definitions.
    fn def_rec(&mut self, name: impl ToString, value: Expr) {
        let value = abs([name.to_string()], value);
        self.already_defined.insert(name.to_string());
        self.defs.push((name.to_string(), value));
    }

    pub fn generate(self, b: &mut LetExprBuilder) {
        self.num.generate(b);

        for (name, value) in self.defs.into_iter() {
            b.def(name, value);
        }
    }
}
