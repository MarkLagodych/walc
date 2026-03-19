//! Implements bitwise AND, OR, XOR, and NOT, shifts, rotations, counting leading/trailing
//! zeroes and counting ones.

use super::*;

/// Applies bit NOT to every bit
pub fn invert(rt: &mut RuntimeGenerator, a: number::Number) -> number::Number {
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

/// Applies a bitwise operation.
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

/// Adds 2^n zeroes to the little end of the number.
/// This is equivalent to a single step of shifting to the left (in BE terms).
///
/// Example:
/// ```text
/// a      = 10101010     (LE binary)
/// n      = 2            (decimal)
/// result = 000010101010 (LE binary)
///          ^^^^
///    2^n zeroes are added
/// ```
fn add_trailing_zeroes(rt: &mut RuntimeGenerator, a: number::Number, n_log2: u8) -> number::Number {
    let name = format!("_ATZ{n_log2}");
    if !rt.has(&name) {
        let body = match n_log2 {
            0..=3 => {
                let mut result = var("a");

                for _ in 0..(1 << n_log2) {
                    result = list::node(bit(false), result);
                }

                result
            }
            _ => {
                let mut result = var("a");
                result = add_trailing_zeroes(rt, result, n_log2 - 1);
                result = add_trailing_zeroes(rt, result, n_log2 - 1);
                result
            }
        };

        rt.def(&name, abs(["a"], body));
    }

    apply(var(name), [a])
}

/// Drops 2^n bits from the little end of the number.
/// This is equivalent to a single step of shifting to the right (in BE terms).
///
/// Example:
/// ```text
///     2^n bits are dropped
///          vvvv
/// a      = 11110101  (LE binary)
/// n      = 2         (decimal)
/// result = 0101      (LE binary)
/// ```
fn drop_trailing_bits(rt: &mut RuntimeGenerator, a: number::Number, n_log2: u8) -> number::Number {
    let name = format!("_DTB{n_log2}");
    if !rt.has(&name) {
        let body = match n_log2 {
            0..=3 => {
                let mut result = var("a");

                for _ in 0..(1 << n_log2) {
                    result = list::get_tail(result);
                }

                result
            }
            _ => {
                let mut result = var("a");
                result = drop_trailing_bits(rt, result, n_log2 - 1);
                result = drop_trailing_bits(rt, result, n_log2 - 1);
                result
            }
        };

        rt.def(&name, abs(["a"], body));
    }

    apply(var(name), [a])
}

/// `op` must be one of: "SHL", "SHR_U", "SHR_S"
/// `bits` must be one of: 32, 64.
fn apply_shift(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    shift: number::Number,
    op: &str,
    bits: u8,
) -> number::Number {
    let name = format!("{op}{bits}");

    if !rt.has(&name) {
        let body = {
            let mut b = LetExprBuilder::new();

            // This computes log2(bits).
            // A number with `bits` bits can be shifted by `log2(bits)` shift,
            // i.e. a 32-bit number is shifted with a 5-bit number, and for 64 it's 6-bit number.
            let shift_significant_bits = bits.trailing_zeros() as u8;

            for i in 0..shift_significant_bits {
                let shift_bit = list::get_head(var("shift"));

                let shifted = match op {
                    "SHL" => add_trailing_zeroes(rt, var("a"), i),
                    _ => drop_trailing_bits(rt, var("a"), i),
                };

                b.def("a", select(shift_bit, var("a"), shifted));

                if i != shift_significant_bits - 1 {
                    b.def("shift", list::get_tail(var("shift")));
                }
            }

            let result = match op {
                "SHL" => match bits {
                    32 => super::conversions::wrap_i32(rt, var("a")),
                    64 => super::conversions::wrap_i64(rt, var("a")),
                    _ => unreachable!(),
                },
                "SHR_U" => match bits {
                    32 => super::conversions::widen_i32(rt, var("a")),
                    64 => super::conversions::widen_i64(rt, var("a")),
                    _ => unreachable!(),
                },
                "SHR_S" => match bits {
                    32 => super::conversions::widen_and_sign_extend_i32(rt, var("a")),
                    64 => super::conversions::widen_and_sign_extend_i64(rt, var("a")),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            };

            b.build_in(result)
        };

        rt.def(&name, abs(["a", "shift"], body));
    }

    apply(var(name), [a, shift])
}

pub fn i32_shift_left(
    rt: &mut RuntimeGenerator,
    a: number::I32,
    shift: number::Number,
) -> number::I32 {
    apply_shift(rt, a, shift, "SHL", 32)
}

pub fn i32_shift_right_unsigned(
    rt: &mut RuntimeGenerator,
    a: number::I32,
    shift: number::Number,
) -> number::I32 {
    apply_shift(rt, a, shift, "SHR_U", 32)
}

pub fn i32_shift_right_signed(
    rt: &mut RuntimeGenerator,
    a: number::I32,
    shift: number::Number,
) -> number::I32 {
    apply_shift(rt, a, shift, "SHR_S", 32)
}

pub fn i64_shift_left(
    rt: &mut RuntimeGenerator,
    a: number::I64,
    shift: number::Number,
) -> number::I64 {
    apply_shift(rt, a, shift, "SHL", 64)
}

pub fn i64_shift_right_unsigned(
    rt: &mut RuntimeGenerator,
    a: number::I64,
    shift: number::Number,
) -> number::I64 {
    apply_shift(rt, a, shift, "SHR_U", 64)
}

pub fn i64_shift_right_signed(
    rt: &mut RuntimeGenerator,
    a: number::I64,
    shift: number::Number,
) -> number::I64 {
    apply_shift(rt, a, shift, "SHR_S", 64)
}

/// `op` must be one of: "ROTR", "ROTL".
/// `bits` must be one of: 32, 64.
fn rotate(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    shift: number::Number,
    op: &str,
    bits: u8,
) -> number::Number {
    let name = format!("{op}{bits}");
    if !rt.has(&name) {
        let body = {
            let shifted_part = match (op, bits) {
                ("ROTR", 32) => i32_shift_right_unsigned(rt, var("a"), var("shift")),
                ("ROTR", 64) => i64_shift_right_unsigned(rt, var("a"), var("shift")),
                ("ROTL", 32) => i32_shift_left(rt, var("a"), var("shift")),
                ("ROTL", 64) => i64_shift_left(rt, var("a"), var("shift")),
                _ => unreachable!(),
            };

            // This is equivalent to `32|64 - shift` because 32|64 is a power of 2 and
            // only a few lower bits matter.
            let minus_shift = super::arithmetic::negate(rt, var("shift"));

            let rotated_part = match (op, bits) {
                ("ROTR", 32) => i32_shift_left(rt, var("a"), minus_shift),
                ("ROTR", 64) => i64_shift_left(rt, var("a"), minus_shift),
                ("ROTL", 32) => i32_shift_right_unsigned(rt, var("a"), minus_shift),
                ("ROTL", 64) => i64_shift_right_unsigned(rt, var("a"), minus_shift),
                _ => unreachable!(),
            };

            or(rt, shifted_part, rotated_part)
        };

        rt.def(&name, abs(["a", "shift"], body));
    }

    apply(var(name), [a, shift])
}

pub fn i32_rotate_right(
    rt: &mut RuntimeGenerator,
    a: number::I32,
    shift: number::Number,
) -> number::I32 {
    rotate(rt, a, shift, "ROTR", 32)
}

pub fn i32_rotate_left(
    rt: &mut RuntimeGenerator,
    a: number::I32,
    shift: number::Number,
) -> number::I32 {
    rotate(rt, a, shift, "ROTL", 32)
}

pub fn i64_rotate_right(
    rt: &mut RuntimeGenerator,
    a: number::I64,
    shift: number::Number,
) -> number::I64 {
    rotate(rt, a, shift, "ROTR", 64)
}

pub fn i64_rotate_left(
    rt: &mut RuntimeGenerator,
    a: number::I64,
    shift: number::Number,
) -> number::I64 {
    rotate(rt, a, shift, "ROTL", 64)
}

fn count_trailing_zeroes(rt: &mut RuntimeGenerator, a: number::Number, bits: u8) -> number::Number {
    let name = format!("CTZ{bits}");
    if !rt.has(&name) {
        let helper_name = format!("_CTZ{bits}");

        let body = {
            select(
                list::is_not_empty(var("a")),
                var("count"),
                select(
                    list::get_head(var("a")),
                    apply(
                        rec(var(&helper_name)),
                        [super::increment(rt, var("count")), list::get_tail(var("a"))],
                    ),
                    var("count"),
                ),
            )
        };

        rt.def_rec(&helper_name, abs(["count", "a"], body));

        let zero = match bits {
            32 => rt.num.i32_const(0),
            64 => rt.num.i64_const(0),
            _ => unreachable!(),
        };

        rt.def(&name, apply(rec(var(helper_name)), [zero]));
    }

    apply(var(name), [a])
}

pub fn i32_count_trailing_zeroes(rt: &mut RuntimeGenerator, a: number::I32) -> number::I32 {
    count_trailing_zeroes(rt, a, 32)
}

pub fn i64_count_trailing_zeroes(rt: &mut RuntimeGenerator, a: number::I64) -> number::I64 {
    count_trailing_zeroes(rt, a, 64)
}

fn count_leading_zeroes(rt: &mut RuntimeGenerator, a: number::Number, bits: u8) -> number::Number {
    let name = format!("CLZ{bits}");
    if !rt.has(&name) {
        let body = count_trailing_zeroes(rt, number::reverse_bits(var("a")), bits);
        rt.def(&name, abs(["a"], body));
    }

    apply(var(name), [a])
}

pub fn i32_count_leading_zeroes(rt: &mut RuntimeGenerator, a: number::I32) -> number::I32 {
    count_leading_zeroes(rt, a, 32)
}

pub fn i64_count_leading_zeroes(rt: &mut RuntimeGenerator, a: number::I64) -> number::I64 {
    count_leading_zeroes(rt, a, 64)
}

fn count_ones(rt: &mut RuntimeGenerator, a: number::Number, bits: u8) -> number::Number {
    let name = format!("POPCNT{bits}");
    if !rt.has(&name) {
        let helper_name = format!("_POPCNT{bits}");

        let body = {
            select(
                list::is_not_empty(var("a")),
                var("count"),
                select(
                    list::get_head(var("a")),
                    apply(
                        rec(var(&helper_name)),
                        [var("count"), list::get_tail(var("a"))],
                    ),
                    apply(
                        rec(var(&helper_name)),
                        [super::increment(rt, var("count")), list::get_tail(var("a"))],
                    ),
                ),
            )
        };

        rt.def_rec(&helper_name, abs(["count", "a"], body));

        let zero = match bits {
            32 => rt.num.i32_const(0),
            64 => rt.num.i64_const(0),
            _ => unreachable!(),
        };

        rt.def(&name, apply(rec(var(helper_name)), [zero]));
    }

    apply(var(name), [a])
}

pub fn i32_count_ones(rt: &mut RuntimeGenerator, a: number::I32) -> number::I32 {
    count_ones(rt, a, 32)
}

pub fn i64_count_ones(rt: &mut RuntimeGenerator, a: number::I64) -> number::I64 {
    count_ones(rt, a, 64)
}
