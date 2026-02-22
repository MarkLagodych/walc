use crate::codegen::*;

use std::collections::BTreeMap as Map;

#[derive(Default)]
pub struct ArithDefinitionBuilder {
    defs: Map<String, ArithDefinitionFn>,
}

type ArithDefinitionFn = fn(&mut ArithDefinitionContext) -> Expr;

struct ArithDefinitionContext<'a> {
    consts: &'a mut number::ConstantDefinitionBuilder,
}

impl ArithDefinitionBuilder {
    pub fn build(self, consts: &mut number::ConstantDefinitionBuilder) -> DefinitionBuilder {
        let mut b = DefinitionBuilder::new();

        let mut ctx = ArithDefinitionContext { consts };

        for (def_name, def) in self.defs.into_iter() {
            b.def(def_name, def(&mut ctx));
        }

        b
    }

    fn def(&mut self, name: impl ToString, def: ArithDefinitionFn) {
        self.defs.insert(name.to_string(), def);
    }

    /// Bit inversion
    pub fn bit_inv(&mut self, a: Bit) -> Bit {
        select(a, bit(true), bit(false))
    }

    /// Equal zero: checks if a number is zero.
    pub fn eqz(&mut self, a: number::Number) -> Bit {
        self.def("EQZ", |_| {
            abs(["a"], {
                let mut b = DefinitionBuilder::new();

                b.def(
                    "eqz",
                    abs(["eqz", "a"], {
                        select(
                            list::is_not_empty(var("a")),
                            bit(true),
                            select(
                                list::get_head(var("a")),
                                apply(rec(var("eqz")), [list::get_tail(var("a"))]),
                                bit(false),
                            ),
                        )
                    }),
                );

                b.build(apply(rec(var("eqz")), [var("a")]))
            })
        });

        apply(var("EQZ"), [a])
    }

    /// Not equal zero
    pub fn neqz(&mut self, a: number::Number) -> Bit {
        let eqz_a = self.eqz(a);
        self.bit_inv(eqz_a)
    }
}
