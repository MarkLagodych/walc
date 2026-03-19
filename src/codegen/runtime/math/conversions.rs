//! Implements integer wrapping (truncation) and extension (widening) between integers, and
//! also conversions between integers and bytes or IDs.

use super::*;

/// Chops the number into parts using the template and returns a big-endian list of the
/// parts.
///
/// `template` is a number where a 1-bit indicates an end of a part.
/// This means that there will be as many parts as there are 1-bits in the template.
/// It *can be longer* than the number being chopped, but no shorter.
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
fn chop(rt: &mut RuntimeGenerator, a: number::Number, template: number::Number) -> list::List {
    if !rt.has("CHOP") {
        let a_head = list::get_head(var("a"));
        let a_tail = list::get_tail(var("a"));

        let template_head = list::get_head(var("template"));
        let template_tail = list::get_tail(var("template"));

        let new_part = list::node(a_head, var("part"));

        let body = abs(
            ["parts", "part", "a", "template"],
            select(
                list::is_not_empty(var("template")),
                var("parts"),
                select(
                    template_head,
                    apply(
                        rec(var("_CHOP")),
                        [
                            var("parts"),
                            new_part.clone(),
                            a_tail.clone(),
                            template_tail.clone(),
                        ],
                    ),
                    apply(
                        rec(var("_CHOP")),
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

        rt.def_rec("_CHOP", body);

        rt.def(
            "CHOP",
            apply(rec(var("_CHOP")), [list::empty(), list::empty()]),
        );
    }

    apply(var("CHOP"), [a, template])
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

/// Wraps the bits at a limit of 32 bits.
pub fn wrap_i32(rt: &mut RuntimeGenerator, a: number::Number) -> number::I32 {
    let template = rt.num.i32_const(1 << 31);
    let parts = chop(rt, a, template);
    // Convert it back to a little-endian number
    number::reverse_bits(list::get_head(parts))
}

/// Wraps the bits at a limit of 64 bits.
pub fn wrap_i64(rt: &mut RuntimeGenerator, a: number::Number) -> number::I64 {
    let template = rt.num.i64_const(1 << 63);
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
/// The template should be filled with zeroes.
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
            var("template"),
            list::node(a_head, apply(rec(var("_WIDEN")), [a_tail, template_tail])),
        );

        rt.def_rec("_WIDEN", abs(["a", "template"], body));

        rt.def("WIDEN", rec(var("_WIDEN")));
    }

    apply(var("WIDEN"), [a, template])
}

/// Widens a given list of bits to a I32
pub fn widen_i32(rt: &mut RuntimeGenerator, a: list::List) -> number::I32 {
    let template = rt.num.i32_const(0);
    widen(rt, a, template)
}

/// Widens a given list of bits to a I64
pub fn widen_i64(rt: &mut RuntimeGenerator, a: list::List) -> number::I64 {
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

/// Widen a number to match the width of the template by copying the sign bit.
///
/// The template must not be shorter than the number given.
/// Its bits are ignored.
///
/// Example:
/// ```text
///         the sign bit....
///               v
/// a        = 0101     (LE)
/// template = 00000000 (LE)
/// result   = 01011111 (LE)
///                ^^^^
///       ...gets copied here
/// ```
fn widen_and_sign_extend_with_template(
    rt: &mut RuntimeGenerator,
    a: list::List,
    template: number::Number,
) -> number::Number {
    if !rt.has("WSGNEXT") {
        let definition = {
            abs(["bit", "a", "template"], {
                let a_not_empty = list::is_not_empty(var("a"));
                let a_head = list::get_head(var("a"));
                let a_tail = list::get_tail(var("a"));

                let template_not_empty = list::is_not_empty(var("template"));
                let template_tail = list::get_tail(var("template"));

                select(
                    template_not_empty,
                    var("a"),
                    select(
                        a_not_empty,
                        list::node(
                            var("bit"),
                            apply(
                                rec(var("_WSGNEXT")),
                                [var("bit"), list::empty(), template_tail.clone()],
                            ),
                        ),
                        list::node(
                            a_head.clone(),
                            apply(rec(var("_WSGNEXT")), [a_head, a_tail, template_tail]),
                        ),
                    ),
                )
            })
        };

        rt.def_rec("_WSGNEXT", definition);

        rt.def("WSGNEXT", apply(rec(var("_WSGNEXT")), [bit(false)]));
    }

    apply(var("WSGNEXT"), [a, template])
}

pub fn widen_and_sign_extend_i32(rt: &mut RuntimeGenerator, a: list::List) -> number::I32 {
    let template = rt.num.i32_const(0);
    widen_and_sign_extend_with_template(rt, a, template)
}

pub fn widen_and_sign_extend_i64(rt: &mut RuntimeGenerator, a: list::List) -> number::I64 {
    let template = rt.num.i64_const(0);
    widen_and_sign_extend_with_template(rt, a, template)
}
