use super::*;

impl UtilGenerator {
    fn bit_not(&self, a: Bit) -> Bit {
        select(a, bit(true), bit(false))
    }

    fn bit_and(&self, a: Bit, b: Bit) -> Bit {
        select(a, bit(false), b)
    }

    fn bit_or(&self, a: Bit, b: Bit) -> Bit {
        select(a, b, bit(true))
    }

    fn bit_less(&self, a: Bit, b: Bit) -> Bit {
        select(a, b, bit(false))
    }

    fn bit_less_equal(&self, a: Bit, b: Bit) -> Bit {
        select(a, bit(true), b)
    }

    fn bit_xor(&self, a: Bit, b: Bit) -> Bit {
        select(a, b.clone(), self.bit_not(b))
    }

    fn bit_equal(&self, a: Bit, b: Bit) -> Bit {
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

    /// Only numbers of the same bit length can be compared.
    pub fn num_not_equal(&mut self, a: number::Number, b: number::Number) -> Bit {
        let a_is_equal_to_b = self.num_equal(a, b);
        self.bit_not(a_is_equal_to_b)
    }

    pub fn num_and(&mut self, a: number::Number, b: number::Number) -> number::Number {
        if !self.has("AND") {
            let a_head = list::get_head(var("a"));
            let a_tail = list::get_tail(var("a"));

            let b_head = list::get_head(var("b"));
            let b_tail = list::get_tail(var("b"));

            let heads_and = self.bit_and(a_head, b_head);

            self.def_rec(
                "_AND",
                abs(["a", "b"], {
                    select(
                        list::is_not_empty(var("a")),
                        list::empty(),
                        list::node(heads_and, apply(rec(var("_AND")), [a_tail, b_tail])),
                    )
                }),
            );

            self.def("AND", rec(var("_AND")));
        }

        apply(var("AND"), [a, b])
    }

    pub fn num_or(&mut self, a: number::Number, b: number::Number) -> number::Number {
        if !self.has("OR") {
            let a_head = list::get_head(var("a"));
            let a_tail = list::get_tail(var("a"));

            let b_head = list::get_head(var("b"));
            let b_tail = list::get_tail(var("b"));

            let heads_or = self.bit_or(a_head, b_head);

            self.def_rec(
                "_OR",
                abs(["a", "b"], {
                    select(
                        list::is_not_empty(var("a")),
                        list::empty(),
                        list::node(heads_or, apply(rec(var("_OR")), [a_tail, b_tail])),
                    )
                }),
            );

            self.def("OR", rec(var("_OR")));
        }

        apply(var("OR"), [a, b])
    }

    pub fn num_xor(&mut self, a: number::Number, b: number::Number) -> number::Number {
        if !self.has("XOR") {
            let a_head = list::get_head(var("a"));
            let a_tail = list::get_tail(var("a"));

            let b_head = list::get_head(var("b"));
            let b_tail = list::get_tail(var("b"));

            let heads_xor = self.bit_xor(a_head, b_head);

            self.def_rec(
                "_XOR",
                abs(["a", "b"], {
                    select(
                        list::is_not_empty(var("a")),
                        list::empty(),
                        list::node(heads_xor, apply(rec(var("_XOR")), [a_tail, b_tail])),
                    )
                }),
            );

            self.def("XOR", rec(var("_XOR")));
        }

        apply(var("XOR"), [a, b])
    }

    /// Compares two BE numbers
    fn num_less_unsigned_be(&mut self, a: number::Number, b: number::Number) -> Bit {
        if !self.has("_LT") {
            let a_head = list::get_head(var("a"));
            let a_tail = list::get_tail(var("a"));

            let b_head = list::get_head(var("b"));
            let b_tail = list::get_tail(var("b"));

            let a_less_than_b = self.bit_less(a_head, b_head);

            self.def_rec(
                "_LT",
                abs(["a", "b"], {
                    select(
                        list::is_not_empty(var("a")),
                        bit(true),
                        select(
                            a_less_than_b,
                            bit(false),
                            apply(rec(var("_LT")), [a_tail, b_tail]),
                        ),
                    )
                }),
            );
        }

        apply(var("_LT"), [a, b])
    }

    pub fn num_less_unsigned(&mut self, a: number::Number, b: number::Number) -> Bit {
        if !self.has("LT") {
            let a_be = number::reverse_bits(var("a"));
            let b_be = number::reverse_bits(var("b"));
            let a_less_than_b = self.num_less_unsigned_be(a_be, b_be);

            self.def("LT", abs(["a", "b"], a_less_than_b));
        }

        apply(var("LT"), [a, b])
    }

    fn num_less_equal_unsigned_be(&mut self, a: number::Number, b: number::Number) -> Bit {
        if !self.has("_LE") {
            let a_head = list::get_head(var("a"));
            let a_tail = list::get_tail(var("a"));

            let b_head = list::get_head(var("b"));
            let b_tail = list::get_tail(var("b"));

            let a_less_than_b = self.bit_less_equal(a_head, b_head);

            self.def_rec(
                "_LE",
                abs(["a", "b"], {
                    select(
                        list::is_not_empty(var("a")),
                        bit(true),
                        select(
                            a_less_than_b,
                            bit(false),
                            apply(rec(var("_LE")), [a_tail, b_tail]),
                        ),
                    )
                }),
            );
        }

        apply(var("_LE"), [a, b])
    }

    pub fn num_less_equal_unsigned(&mut self, a: number::Number, b: number::Number) -> Bit {
        if !self.has("LE") {
            let a_be = number::reverse_bits(var("a"));
            let b_be = number::reverse_bits(var("b"));
            let a_less_equal_b = self.num_less_equal_unsigned_be(a_be, b_be);

            self.def("LE", abs(["a", "b"], a_less_equal_b));
        }

        apply(var("LE"), [a, b])
    }

    pub fn num_greater_unsigned(&mut self, a: number::Number, b: number::Number) -> Bit {
        let a_less_equal_b = self.num_less_equal_unsigned(a, b);
        self.bit_not(a_less_equal_b)
    }

    pub fn num_greater_equal_unsigned(&mut self, a: number::Number, b: number::Number) -> Bit {
        let a_less_than_b = self.num_less_unsigned(a, b);
        self.bit_not(a_less_than_b)
    }

    pub fn num_less_signed(&mut self, a: number::Number, b: number::Number) -> Bit {
        if !self.has("LTs") {
            let mut b = LetExprBuilder::new();

            b.def("a_be", number::reverse_bits(var("a")));
            b.def("b_be", number::reverse_bits(var("b")));

            let a_head = list::get_head(var("a_be"));
            let a_tail = list::get_tail(var("a_be"));

            let b_head = list::get_head(var("b_be"));
            let b_tail = list::get_tail(var("b_be"));

            // | A    | B    | A < B     |
            // | ---- | ---- | --------- |
            // | 0aaa | 0bbb | aaa < bbb |
            // | 0aaa | 1bbb | false     |
            // | 1aaa | 0bbb | true      |
            // | 1aaa | 1bbb | aaa > bbb |

            let body = select(
                a_head.clone(),
                select(
                    b_head.clone(),
                    self.num_less_unsigned_be(a_tail.clone(), b_tail.clone()),
                    bit(false),
                ),
                select(b_head, bit(true), self.num_less_unsigned_be(b_tail, a_tail)),
            );

            self.def("LTs", abs(["a", "b"], b.build_in(body)));
        }

        apply(var("LTs"), [a, b])
    }

    pub fn num_less_equal_signed(&mut self, a: number::Number, b: number::Number) -> Bit {
        if !self.has("LEs") {
            let mut b = LetExprBuilder::new();

            b.def("a_be", number::reverse_bits(var("a")));
            b.def("b_be", number::reverse_bits(var("b")));

            let a_head = list::get_head(var("a_be"));
            let a_tail = list::get_tail(var("a_be"));

            let b_head = list::get_head(var("b_be"));
            let b_tail = list::get_tail(var("b_be"));

            // | A    | B    | A <= B     |
            // | ---- | ---- | ---------- |
            // | 0aaa | 0bbb | aaa <= bbb |
            // | 0aaa | 1bbb | false      |
            // | 1aaa | 0bbb | true       |
            // | 1aaa | 1bbb | aaa >= bbb |

            let body = select(
                a_head.clone(),
                select(
                    b_head.clone(),
                    self.num_less_equal_unsigned_be(a_tail.clone(), b_tail.clone()),
                    bit(false),
                ),
                select(
                    b_head,
                    bit(true),
                    self.num_less_equal_unsigned_be(b_tail, a_tail),
                ),
            );

            self.def("LEs", abs(["a", "b"], b.build_in(body)));
        }

        apply(var("LEs"), [a, b])
    }

    pub fn num_greater_signed(&mut self, a: number::Number, b: number::Number) -> Bit {
        let a_less_equal_b = self.num_less_equal_signed(a, b);
        self.bit_not(a_less_equal_b)
    }

    pub fn num_greater_equal_signed(&mut self, a: number::Number, b: number::Number) -> Bit {
        let a_less_than_b = self.num_less_signed(a, b);
        self.bit_not(a_less_than_b)
    }

    pub fn num_add(&mut self, a: number::Number, b: number::Number) -> number::Number {
        if !self.has("ADD") {
            // Full adder for two bits and a carry-in:
            // In: A, B, Cin (carry in)
            // Out: Sum, Cout (carry out)
            // 1. X = A xor B
            // 2. Sum = X xor Cin
            // 3. Cout = (A and B) or (X and Cin)

            let mut b = LetExprBuilder::new();

            let a_head = list::get_head(var("a"));
            let a_tail = list::get_tail(var("a"));

            let b_head = list::get_head(var("b"));
            let b_tail = list::get_tail(var("b"));

            b.def("x", self.bit_xor(a_head.clone(), b_head.clone()));

            let sum_head = self.bit_xor(var("x"), var("c"));

            let carry_out = self.bit_or(
                self.bit_and(a_head, b_head),
                self.bit_and(var("x"), var("c")),
            );

            let sum_tail = apply(rec(var("_ADD")), [carry_out, a_tail, b_tail]);

            let body = select(
                list::is_not_empty(var("a")),
                list::empty(),
                list::node(sum_head, sum_tail),
            );

            self.def_rec("_ADD", abs(["c", "a", "b"], b.build_in(body)));

            self.def("ADD", apply(rec(var("_ADD")), [bit(false)]));
        }

        apply(var("ADD"), [a, b])
    }

    /// Fast algorithm for adding 1 to a number.
    pub fn num_increment(&mut self, a: number::Number) -> number::Number {
        if !self.has("INC") {
            // Adding 1 is equivalent to flipping leading ones and the first zero bit. For example:
            //
            // INC(000000_LE) = 100000_LE
            //     *            *
            //
            // INC(111000_LE) = 000100_LE
            //     ^^^*         ^^^*
            //
            // INC(111111_LE) = 000000_LE
            //     ^^^^^^       ^^^^^^

            let a_head = list::get_head(var("a"));
            let a_tail = list::get_tail(var("a"));

            let body = select(
                list::is_not_empty(var("a")),
                list::empty(),
                select(
                    a_head,
                    list::node(bit(true), a_tail.clone()),
                    list::node(bit(false), apply(rec(var("_INC")), [a_tail])),
                ),
            );

            self.def_rec("_INC", abs(["a"], body));

            self.def("INC", rec(var("_INC")));
        }

        apply(var("INC"), [a])
    }

    /// Applies bit NOT to every bit
    fn num_invert(&mut self, a: number::Number) -> number::Number {
        if !self.has("INV") {
            let a_head = list::get_head(var("a"));
            let a_tail = list::get_tail(var("a"));

            let body = select(
                list::is_not_empty(var("a")),
                list::empty(),
                list::node(self.bit_not(a_head), apply(rec(var("_INV")), [a_tail])),
            );

            self.def_rec("_INV", abs(["a"], body));

            self.def("INV", rec(var("_INV")));
        }

        apply(var("INV"), [a])
    }

    fn num_negate(&mut self, a: number::Number) -> number::Number {
        let a_inverted = self.num_invert(a);
        self.num_increment(a_inverted)
    }

    pub fn num_sub(&mut self, a: number::Number, b: number::Number) -> number::Number {
        let b_negated = self.num_negate(b);
        self.num_add(a, b_negated)
    }
}
