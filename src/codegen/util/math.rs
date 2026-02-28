use super::*;

impl UtilGenerator {
    pub fn bit_not(&mut self, a: Bit) -> Bit {
        select(a, bit(true), bit(false))
    }

    pub fn bit_equal(&mut self, a: Bit, b: Bit) -> Bit {
        select(a, self.bit_not(b.clone()), b)
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

            self.def("EQZ", rec(var("_EQZ")));
        }

        apply(var("EQZ"), [a])
    }

    pub fn num_is_not_zero(&mut self, a: number::Number) -> Bit {
        let a_is_zero = self.num_is_zero(a);
        self.bit_not(a_is_zero)
    }

    /// Only numbers of the same bit length can be compared.
    pub fn num_equal(&mut self, a: number::Number, b: number::Number) -> Bit {
        if !self.has("EQ") {
            let a_head = list::get_head(var("a"));
            let a_tail = list::get_tail(var("a"));

            let b_head = list::get_head(var("b"));
            let b_tail = list::get_tail(var("b"));

            let heads_equal = self.bit_equal(a_head, b_head);

            self.def_rec(
                "_EQ",
                abs(["a", "b"], {
                    select(
                        list::is_not_empty(var("a")),
                        // a is empty here. If b is not empty (i.e. of different bit length),
                        // this is a logical error.
                        bit(true),
                        select(
                            heads_equal,
                            bit(false),
                            apply(rec(var("_EQ")), [a_tail, b_tail]),
                        ),
                    )
                }),
            );

            self.def("EQ", rec(var("_EQ")));
        }

        apply(var("EQ"), [a, b])
    }

    pub fn num_not_equal(&mut self, a: number::Number, b: number::Number) -> Bit {
        let a_is_equal_to_b = self.num_equal(a, b);
        self.bit_not(a_is_equal_to_b)
    }
}
