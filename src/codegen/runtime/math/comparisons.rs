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

/// Compares two unsigned numbers.
/// `b` must not be shorter than `a`, but can be longer.
///
/// `op` must be one of: "LT", "LE"
fn compare(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number, op: &str) -> Bit {
    let name = op;
    if !rt.has(name) {
        let helper_name = format!("_{op}");

        // Numbers are in LE here.
        //
        // A < B or A <= B ("cmp(A, B)" for short) is computed by:
        //
        // | A        | B        | cmp(result, A, B)             |
        // | -------- | -------- | ----------------------------- |
        // | 0aaaaaaa | 0bbbbbbb | cmp(aaaaaaa, bbbbbbb, result) |
        // | 0aaaaaaa | 1bbbbbbb | cmp(aaaaaaa, bbbbbbb, true)   |
        // | 1aaaaaaa | 0bbbbbbb | cmp(aaaaaaa, bbbbbbb, false)  |
        // | 1aaaaaaa | 1bbbbbbb | cmp(aaaaaaa, bbbbbbb, result) |
        // | empty    | whatever | result                        |
        //
        // To put it in normal language, when the bits are equal, do not change the intermediate
        // result. When they differ, update the result.
        //
        // The initial intermediate result for "<=" is true because if numbers are equal, the
        // overall result must be true.

        let a_head = list::get_head(var("a"));
        let a_tail = list::get_tail(var("a"));

        let b_head = list::get_head(var("b"));
        let b_tail = list::get_tail(var("b"));

        let compare_tails = |result| {
            apply(
                rec(var(&helper_name)),
                [result, a_tail.clone(), b_tail.clone()],
            )
        };

        let body = select(
            list::is_not_empty(var("a")),
            var("res"),
            compare_tails(select(
                a_head,
                select(b_head.clone(), var("res"), bit(true)),
                select(b_head, bit(false), var("res")),
            )),
        );

        rt.def_rec(&helper_name, abs(["res", "a", "b"], body));

        let default_result = match op {
            "LT" => bit(false),
            "LE" => bit(true),
            _ => unreachable!(),
        };

        rt.def(name, apply(rec(var(helper_name)), [default_result]));
    }

    apply(var(name), [a, b])
}

/// Compares two unsigned numbers.
/// `b` must not be shorter than `a`, but can be longer.
pub fn less_unsigned(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> Bit {
    compare(rt, a, b, "LT")
}

/// Compares two unsigned numbers.
/// `b` must not be shorter than `a`, but can be longer.
pub fn less_equal_unsigned(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> Bit {
    compare(rt, a, b, "LE")
}

/// Compares two unsigned numbers.
/// `b` must not be shorter than `a`, but can be longer.
pub fn greater_unsigned(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> Bit {
    bit_not(less_equal_unsigned(rt, a, b))
}

/// Compares two unsigned numbers.
/// `b` must not be shorter than `a`, but can be longer.
pub fn greater_equal_unsigned(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    b: number::Number,
) -> Bit {
    bit_not(less_unsigned(rt, a, b))
}

/// Compares two unsigned BE (i.e. reversed) numbers.
/// The numbers must be of the same bit width.
///
/// `op` must be one of: "LTbe" (for Less Than), "LEbe" (for Less Equal).
fn compare_be(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number, op: &str) -> Bit {
    let name = format!("_{op}");

    if !rt.has(&name) {
        // Numbers are in BE here.
        //
        // | A    | B    | A <= B     |
        // | ---- | ---- | ---------- |
        // | 0aaa | 0bbb | aaa <= bbb |
        // | 0aaa | 1bbb | true       |
        // | 1aaa | 0bbb | false      |
        // | 1aaa | 1bbb | aaa <= bbb |

        let a_head = list::get_head(var("a"));
        let a_tail = list::get_tail(var("a"));

        let b_head = list::get_head(var("b"));
        let b_tail = list::get_tail(var("b"));

        let fallback = match op {
            "LTbe" => bit(false),
            "LEbe" => bit(true),
            _ => unreachable!(),
        };

        let compare_tails = apply(rec(var(&name)), [a_tail.clone(), b_tail.clone()]);

        rt.def_rec(
            &name,
            abs(["a", "b"], {
                select(
                    list::is_not_empty(var("a")),
                    fallback,
                    select(
                        a_head,
                        select(b_head.clone(), compare_tails.clone(), bit(true)),
                        select(b_head, bit(false), compare_tails),
                    ),
                )
            }),
        );
    }

    apply(rec(var(name)), [a, b])
}

/// Compares two unsigned BE (i.e. reversed) numbers.
/// The numbers must be of the same bit width.
fn less_unsigned_be(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> Bit {
    compare_be(rt, a, b, "LTbe")
}

/// Compares two unsigned BE (i.e. reversed) numbers.
/// The numbers must be of the same bit width.
fn less_equal_unsigned_be(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> Bit {
    compare_be(rt, a, b, "LEbe")
}

/// Cmopares two numbers of the same bit width.
/// `op` must be one of: "LTs" (for Less Than), "LEs" (for Less Equal).
fn compare_signed(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    b: number::Number,
    op: &str,
) -> Bit {
    let name = format!("_{op}");
    if !rt.has(&name) {
        // Numbers are in BE here.
        //
        // | A    | B    | A <= B     |
        // | ---- | ---- | ---------- |
        // | 0aaa | 0bbb | aaa <= bbb |
        // | 0aaa | 1bbb | false      |
        // | 1aaa | 0bbb | true       |
        // | 1aaa | 1bbb | aaa <= bbb |

        let mut b = LetExprBuilder::new();

        b.def("a_be", number::reverse_bits(var("a")));
        b.def("b_be", number::reverse_bits(var("b")));

        let a_head = list::get_head(var("a_be"));
        let a_tail = list::get_tail(var("a_be"));

        let b_head = list::get_head(var("b_be"));
        let b_tail = list::get_tail(var("b_be"));

        let compare_tail = match op {
            "LTs" => less_unsigned_be(rt, a_tail.clone(), b_tail.clone()),
            "LEs" => less_equal_unsigned_be(rt, a_tail.clone(), b_tail.clone()),
            _ => unreachable!(),
        };

        let body = select(
            a_head.clone(),
            select(b_head.clone(), compare_tail.clone(), bit(false)),
            select(b_head, bit(true), compare_tail),
        );

        rt.def(&name, abs(["a", "b"], b.build_in(body)));
    }

    apply(var(&name), [a, b])
}

/// The numbers must be of the same bit width.
pub fn less_signed(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> Bit {
    compare_signed(rt, a, b, "LTs")
}

/// The numbers must be of the same bit width.
pub fn less_equal_signed(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> Bit {
    compare_signed(rt, a, b, "LEs")
}

/// The numbers must be of the same bit width.
pub fn greater_signed(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> Bit {
    bit_not(less_equal_signed(rt, a, b))
}

/// The numbers must be of the same bit width.
pub fn greater_equal_signed(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    b: number::Number,
) -> Bit {
    bit_not(less_signed(rt, a, b))
}
