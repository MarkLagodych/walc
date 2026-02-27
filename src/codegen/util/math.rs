use super::*;

impl UtilGenerator {
    pub fn bit_not(&mut self, a: Bit) -> Bit {
        select(a, bit(true), bit(false))
    }

    pub fn num_is_zero(&mut self, a: number::Number) -> Bit {
        if !self.has("EQZ") {
            self.def_rec(
                "_EQZ",
                abs(["a"], {
                    select(
                        list::is_not_empty(var("a")),
                        bit(true),
                        select(
                            list::get_head(var("a")),
                            apply(rec(var("_EQZ")), [list::get_tail(var("a"))]),
                            bit(false),
                        ),
                    )
                }),
            );

            self.def("EQZ", abs(["a"], apply(rec(var("_EQZ")), [var("a")])));
        }

        apply(var("EQZ"), [a])
    }

    pub fn num_is_not_zero(&mut self, a: number::Number) -> Bit {
        let a_is_zero = self.num_is_zero(a);
        self.bit_not(a_is_zero)
    }
}
