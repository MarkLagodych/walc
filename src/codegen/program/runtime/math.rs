use super::*;

use crate::codegen::*;

impl RuntimeGenerator {
    /// Bit inversion
    pub fn bit_inv(&mut self, a: Bit) -> Bit {
        select(a, bit(true), bit(false))
    }

    /// Equal zero: checks if a number is zero.
    pub fn eqz(&mut self, a: number::Number) -> Bit {
        if !self.has("EQZ") {
            self.def(
                "EQZ",
                abs(["a"], {
                    let mut b = LetExprBuilder::new();

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

                    b.build_in(apply(rec(var("eqz")), [var("a")]))
                }),
            );
        }

        apply(var("EQZ"), [a])
    }

    /// Not equal zero
    pub fn neqz(&mut self, a: number::Number) -> Bit {
        let eqz_a = self.eqz(a);
        self.bit_inv(eqz_a)
    }
}
