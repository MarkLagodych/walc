use super::*;

fn bit_not(a: Bit) -> Bit {
    select(a, bit(true), bit(false))
}

fn bit_and(a: Bit, b: Bit) -> Bit {
    select(a, bit(false), b)
}

fn bit_or(a: Bit, b: Bit) -> Bit {
    select(a, b, bit(true))
}

fn bit_less(a: Bit, b: Bit) -> Bit {
    select(a, b, bit(false))
}

fn bit_less_equal(a: Bit, b: Bit) -> Bit {
    select(a, bit(true), b)
}

fn bit_xor(a: Bit, b: Bit) -> Bit {
    select(a, b.clone(), bit_not(b))
}

fn bit_equal(a: Bit, b: Bit) -> Bit {
    select(a, bit_not(b.clone()), b)
}

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
    let a_is_zero = is_zero(rt, a);
    bit_not(a_is_zero)
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

        rt.def("EQ", rec(var("_EQ")));
    }

    apply(var("EQ"), [a, b])
}

/// Only numbers of the same bit length can be compared.
pub fn not_equal(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> Bit {
    let a_is_equal_to_b = equal(rt, a, b);
    bit_not(a_is_equal_to_b)
}

fn apply_bitop(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    b: number::Number,
    op: &str,
) -> number::Number {
    if !rt.has(op) {
        let a_head = list::get_head(var("a"));
        let a_tail = list::get_tail(var("a"));

        let b_head = list::get_head(var("b"));
        let b_tail = list::get_tail(var("b"));

        let head = match op {
            "AND" => bit_and(a_head, b_head),
            "OR" => bit_or(a_head, b_head),
            "XOR" => bit_xor(a_head, b_head),
            _ => unreachable!(),
        };

        rt.def_rec(
            format!("_{op}"),
            abs(["a", "b"], {
                select(
                    list::is_not_empty(var("a")),
                    list::empty(),
                    list::node(head, apply(rec(var(format!("_{op}"))), [a_tail, b_tail])),
                )
            }),
        );

        rt.def(op, rec(var(format!("_{op}"))));
    }

    apply(var(op), [a, b])
}

pub fn and(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> number::Number {
    apply_bitop(rt, a, b, "AND")
}

pub fn or(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> number::Number {
    apply_bitop(rt, a, b, "OR")
}

pub fn xor(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> number::Number {
    apply_bitop(rt, a, b, "XOR")
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
    let a_less_equal_b = less_equal_unsigned(rt, a, b);
    bit_not(a_less_equal_b)
}

pub fn greater_equal_unsigned(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    b: number::Number,
) -> Bit {
    let a_less_than_b = less_unsigned(rt, a, b);
    bit_not(a_less_than_b)
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
    let a_less_equal_b = less_equal_signed(rt, a, b);
    bit_not(a_less_equal_b)
}

pub fn greater_equal_signed(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    b: number::Number,
) -> Bit {
    let a_less_than_b = less_signed(rt, a, b);
    bit_not(a_less_than_b)
}

pub fn add(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> number::Number {
    if !rt.has("ADD") {
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

        b.def("x", bit_xor(a_head.clone(), b_head.clone()));

        let sum_head = bit_xor(var("x"), var("c"));

        let carry_out = bit_or(bit_and(a_head, b_head), bit_and(var("x"), var("c")));

        let sum_tail = apply(rec(var("_ADD")), [carry_out, a_tail, b_tail]);

        let body = select(
            list::is_not_empty(var("a")),
            list::empty(),
            list::node(sum_head, sum_tail),
        );

        rt.def_rec("_ADD", abs(["c", "a", "b"], b.build_in(body)));

        rt.def("ADD", apply(rec(var("_ADD")), [bit(false)]));
    }

    apply(var("ADD"), [a, b])
}

/// Fast algorithm for adding 1 to a number.
pub fn increment(rt: &mut RuntimeGenerator, a: number::Number) -> number::Number {
    if !rt.has("INC") {
        // Adding 1 is equivalent to flipping leading ones and the first zero bit. For example:
        //
        // INC(000000) = 100000   (numbers are in LE)
        //     *         *
        //
        // INC(111000) = 000100   (numbers are in LE)
        //     ^^^*      ^^^*
        //
        // INC(111111) = 000000   (numbers are in LE)
        //     ^^^^^^    ^^^^^^

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

        rt.def_rec("_INC", abs(["a"], body));

        rt.def("INC", rec(var("_INC")));
    }

    apply(var("INC"), [a])
}

/// Applies bit NOT to every bit
fn invert(rt: &mut RuntimeGenerator, a: number::Number) -> number::Number {
    if !rt.has("INV") {
        let a_head = list::get_head(var("a"));
        let a_tail = list::get_tail(var("a"));

        let body = select(
            list::is_not_empty(var("a")),
            list::empty(),
            list::node(bit_not(a_head), apply(rec(var("_INV")), [a_tail])),
        );

        rt.def_rec("_INV", abs(["a"], body));

        rt.def("INV", rec(var("_INV")));
    }

    apply(var("INV"), [a])
}

fn negate(rt: &mut RuntimeGenerator, a: number::Number) -> number::Number {
    let a_inverted = invert(rt, a);
    increment(rt, a_inverted)
}

pub fn sub(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> number::Number {
    let b_negated = negate(rt, b);
    add(rt, a, b_negated)
}

/// Chops the number into parts using the template and returns a big-endian list of the
/// parts.
///
/// The template is a number where a 1-bit indicates an end of a part.
/// This means that there will be as many parts as there are bits in the number.
/// It must not be shorter than the number being chopped, but can be longer.
///
/// The bits of the returned parts are reversed, so if the the input number is in little endian,
/// then the returned parts are in big endian. This is useful e.g. for dividing a number into
/// bytes or converting a number into an `Id`.
///
/// Example:
/// ```text
/// a = ABCDEFGH (LE)
/// template = 00100000 (LE)
/// result = [CBA] (the item is BE)
///
/// a = ABCDEFGH (LE)
/// template = 00100100 (LE)
/// result = [FED, CBA] (list is BE, items are BE)
///
/// a = ABCDEFGH (LE)
/// template = 00100101 (LE)
/// result = [HG, FED, CBA] (list is BE, items are BE)
/// ```
pub fn chop(rt: &mut RuntimeGenerator, a: number::Number, template: number::Number) -> list::List {
    if !rt.has("SEP") {
        let a_head = list::get_head(var("a"));
        let a_tail = list::get_tail(var("a"));

        let template_head = list::get_head(var("template"));
        let template_tail = list::get_tail(var("template"));

        let new_part = list::node(a_head, var("part"));

        let body = abs(
            ["parts", "part", "a", "template"],
            select(
                list::is_not_empty(var("a")),
                var("parts"),
                select(
                    template_head,
                    apply(
                        rec(var("_SEP")),
                        [
                            var("parts"),
                            new_part.clone(),
                            a_tail.clone(),
                            template_tail.clone(),
                        ],
                    ),
                    apply(
                        rec(var("_SEP")),
                        [
                            list::node(new_part.clone(), var("parts")),
                            list::empty(),
                            a_tail,
                            template_tail,
                        ],
                    ),
                ),
            ),
        );

        rt.def_rec("_SEP", body);

        rt.def(
            "SEP",
            apply(rec(var("_SEP")), [list::empty(), list::empty()]),
        );
    }

    apply(var("SEP"), [a, template])
}

/// Takes a number `a` with `source_bits`, takes `target_bits` lowest bits and converts the
/// result to a big-endian list of bytes.
pub fn split_lowest_bits_to_be_bytes(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    source_bits: u8,
    target_bits: u8,
) -> list::List {
    let byte_template = match (source_bits, target_bits) {
        (32, 8) => rt.num.i32_const(0x00_00_00_80),
        (32, 16) => rt.num.i32_const(0x00_00_80_80),
        (32, 32) => rt.num.i32_const(0x80_80_80_80),
        (64, 8) => rt.num.i64_const(0x00_00_00_00_00_00_00_80),
        (64, 16) => rt.num.i64_const(0x00_00_00_00_00_00_80_80),
        (64, 32) => rt.num.i64_const(0x00_00_00_00_80_80_80_80),
        (64, 64) => rt.num.i64_const(0x80_80_80_80_80_80_80_80),
        _ => unreachable!(),
    };

    chop(rt, a, byte_template)
}

pub fn i64_to_i32(rt: &mut RuntimeGenerator, a: number::I64) -> number::I32 {
    // Cut the number in half (only the lower part is kept)
    let template = rt.num.i64_const(1 << 31);
    let parts = chop(rt, a, template);
    // Convert it back to a little-endian number
    number::reverse_bits(list::get_head(parts))
}

pub fn i32_to_id(rt: &mut RuntimeGenerator, a: number::I32) -> number::Id {
    // Cut the number in half (only the lower part is kept)
    let template = rt.num.i32_const(1 << 15);
    let parts = chop(rt, a, template);
    list::get_head(parts)
}

pub fn i32_to_byte(rt: &mut RuntimeGenerator, a: number::I32) -> number::Byte {
    // Get the lowest byte of the value
    let bytes = split_lowest_bits_to_be_bytes(rt, a, 32, 8);
    list::get_head(bytes)
}

/// Widens a number by copying the missing bits from the template.
/// The template must not be shorter than the number being widened.
/// It is intended that the template is filled with zeroes.
///
/// Example:
/// ```text
/// a        = 1010 (LE)
/// template = 00000000 (LE)
/// result   = 10100000 (LE)
///                ^^^^
///    these bits are copied from the template
/// ```
fn widen(rt: &mut RuntimeGenerator, a: number::Number, template: number::Number) -> number::Number {
    if !rt.has("WIDEN") {
        let a_head = list::get_head(var("a"));
        let a_tail = list::get_tail(var("a"));

        let template_tail = list::get_tail(var("template"));

        let body = select(
            list::is_not_empty(var("a")),
            list::node(a_head, apply(rec(var("_WIDEN")), [a_tail, template_tail])),
            var("template"),
        );

        rt.def_rec("_WIDEN", abs(["a", "template"], body));

        rt.def(
            "WIDEN",
            apply(rec(var("_WIDEN")), [var("a"), var("template")]),
        );
    }

    apply(var("WIDEN"), [a, template])
}

pub fn i32_to_i64(rt: &mut RuntimeGenerator, a: number::I32) -> number::I64 {
    let template = rt.num.i64_const(0);
    widen(rt, a, template)
}

/// Copies a single bit of `a` into the bits masked by `extension_mask`.
/// The mask must not be shorter than the number being sign-extended.
///
/// Example 1:
/// ```text
///              sign bit...
///                 v
/// a      = 01010101_00000000 (LE)
/// mask   = 00000000_11111111 (LE)
/// result = 01010101_11111111 (LE)
///                   ^^^^^^^^
///             ...gets copied here
/// ```
///
/// Example 2:
///
/// ```text
///              sign bit...
///                 v
/// a      = 01010100_00000000 (LE)
/// mask   = 00000000_11111111 (LE)
/// result = 01010100_00000000 (LE)
///                   ^^^^^^^^
///             ...gets copied here
/// ```
fn sign_extend_with_mask(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    extension_mask: number::Number,
) -> number::Number {
    if !rt.has("SGNEXT") {
        let definition = {
            abs(["bit", "a", "mask"], {
                let a_head = list::get_head(var("a"));
                let a_tail = list::get_tail(var("a"));

                let mask_head = list::get_head(var("mask"));
                let mask_tail = list::get_tail(var("mask"));

                select(
                    list::is_not_empty(var("a")),
                    list::empty(),
                    select(
                        mask_head,
                        list::node(
                            a_head.clone(),
                            apply(
                                rec(var("_SGNEXT")),
                                [a_head, a_tail.clone(), mask_tail.clone()],
                            ),
                        ),
                        list::node(
                            var("bit"),
                            apply(rec(var("_SGNEXT")), [var("bit"), a_tail, mask_tail]),
                        ),
                    ),
                )
            })
        };

        rt.def_rec("_SGNEXT", definition);

        rt.def("SGNEXT", apply(rec(var("_SGNEXT")), [bit(false)]));
    }

    apply(var("SGNEXT"), [a, extension_mask])
}

pub fn sign_extend(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    target_bits: u8,
    source_bits: u8,
) -> number::Number {
    let extension_mask = match (target_bits, source_bits) {
        (32, 8) => rt.num.i32_const(0xff_ff_ff_00),
        (32, 16) => rt.num.i32_const(0xff_ff_00_00),
        (64, 8) => rt.num.i64_const(0xff_ff_ff_ff_ff_ff_ff_00),
        (64, 16) => rt.num.i64_const(0xff_ff_ff_ff_ff_ff_00_00),
        (64, 32) => rt.num.i64_const(0xff_ff_ff_ff_00_00_00_00),
        _ => unreachable!(),
    };

    sign_extend_with_mask(rt, a, extension_mask)
}
