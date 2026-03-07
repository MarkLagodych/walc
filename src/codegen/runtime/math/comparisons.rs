use super::*;

pub fn is_zero(rt: &mut RuntimeGenerator, a: number::Number) -> Bit {
    if !rt.has("EQZ") {
        rt.def_rec(
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

        rt.def("EQZ", rec(var("_EQZ")));
    }

    apply(var("EQZ"), [a])
}

pub fn is_not_zero(rt: &mut RuntimeGenerator, a: number::Number) -> Bit {
    bit_not(is_zero(rt, a))
}

/// Only numbers of the same bit length can be compared.
pub fn equal(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> Bit {
    if !rt.has("EQ") {
        let a_head = list::get_head(var("a"));
        let a_tail = list::get_tail(var("a"));

        let b_head = list::get_head(var("b"));
        let b_tail = list::get_tail(var("b"));

        let heads_equal = bit_equal(a_head, b_head);

        rt.def_rec(
            "_EQ",
            abs(["a", "b"], {
                select(
                    list::is_not_empty(var("a")),
                    bit(true),
                    select(
                        heads_equal,
                        bit(false),
                        apply(rec(var("_EQ")), [a_tail, b_tail]),
                    ),
                )
            }),
        );

        rt.def("EQ", rec(var("_EQ")));
    }

    apply(var("EQ"), [a, b])
}

/// Only numbers of the same bit length can be compared.
pub fn not_equal(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> Bit {
    let a_is_equal_to_b = equal(rt, a, b);
    bit_not(a_is_equal_to_b)
}

/// Compares two BE numbers
fn less_unsigned_be(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> Bit {
    if !rt.has("_LT") {
        let a_head = list::get_head(var("a"));
        let a_tail = list::get_tail(var("a"));

        let b_head = list::get_head(var("b"));
        let b_tail = list::get_tail(var("b"));

        let a_less_than_b = bit_less(a_head, b_head);

        rt.def_rec(
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

pub fn less_unsigned(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> Bit {
    if !rt.has("LT") {
        let a_be = number::reverse_bits(var("a"));
        let b_be = number::reverse_bits(var("b"));
        let a_less_than_b = less_unsigned_be(rt, a_be, b_be);

        rt.def("LT", abs(["a", "b"], a_less_than_b));
    }

    apply(var("LT"), [a, b])
}

fn less_equal_unsigned_be(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> Bit {
    if !rt.has("_LE") {
        let a_head = list::get_head(var("a"));
        let a_tail = list::get_tail(var("a"));

        let b_head = list::get_head(var("b"));
        let b_tail = list::get_tail(var("b"));

        let a_less_than_b = bit_less_equal(a_head, b_head);

        rt.def_rec(
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

pub fn less_equal_unsigned(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> Bit {
    if !rt.has("LE") {
        let a_be = number::reverse_bits(var("a"));
        let b_be = number::reverse_bits(var("b"));
        let a_less_equal_b = less_equal_unsigned_be(rt, a_be, b_be);

        rt.def("LE", abs(["a", "b"], a_less_equal_b));
    }

    apply(var("LE"), [a, b])
}

pub fn greater_unsigned(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> Bit {
    bit_not(less_equal_unsigned(rt, a, b))
}

pub fn greater_equal_unsigned(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    b: number::Number,
) -> Bit {
    bit_not(less_unsigned(rt, a, b))
}

pub fn less_signed(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> Bit {
    if !rt.has("LTs") {
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
                less_unsigned_be(rt, a_tail.clone(), b_tail.clone()),
                bit(false),
            ),
            select(b_head, bit(true), less_unsigned_be(rt, b_tail, a_tail)),
        );

        rt.def("LTs", abs(["a", "b"], b.build_in(body)));
    }

    apply(var("LTs"), [a, b])
}

pub fn less_equal_signed(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> Bit {
    if !rt.has("LEs") {
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
                less_equal_unsigned_be(rt, a_tail.clone(), b_tail.clone()),
                bit(false),
            ),
            select(
                b_head,
                bit(true),
                less_equal_unsigned_be(rt, b_tail, a_tail),
            ),
        );

        rt.def("LEs", abs(["a", "b"], b.build_in(body)));
    }

    apply(var("LEs"), [a, b])
}

pub fn greater_signed(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> Bit {
    bit_not(less_equal_signed(rt, a, b))
}

pub fn greater_equal_signed(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    b: number::Number,
) -> Bit {
    bit_not(less_signed(rt, a, b))
}
