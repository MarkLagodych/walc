use super::*;

pub fn push_const(rt: &mut RuntimeGenerator, op: &Operator) -> Instruction {
    if !rt.has("Push") {
        rt.def("Push", {
            abs(["item"], {
                let mut b = InstructionBuilder::new();
                b.push([var("item")]);
                b.build()
            })
        });
    }

    let item = rt.num.with_init_value(op);
    apply(var("Push"), [item])
}

pub fn eqz(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("Eqz") {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a"]);

            let result = select(
                math::is_zero(rt, var("a")),
                rt.num.i32_const(0),
                rt.num.i32_const(1),
            );

            b.push([result]);

            b.build()
        };
        rt.def("Eqz", definition);
    }

    var("Eqz")
}

/// `op` is "Clz32", "Ctz32", "PopCnt32", "Clz64", "Ctz64", "PopCnt64".
fn unop(rt: &mut RuntimeGenerator, op: &str) -> Instruction {
    if !rt.has(op) {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a"]);

            let result = match op {
                "Clz32" => math::i32_count_leading_zeroes(rt, var("a")),
                "Clz64" => math::i64_count_leading_zeroes(rt, var("a")),

                "Ctz32" => math::i32_count_trailing_zeroes(rt, var("a")),
                "Ctz64" => math::i64_count_trailing_zeroes(rt, var("a")),

                "PopCnt32" => math::i32_count_ones(rt, var("a")),
                "PopCnt64" => math::i64_count_ones(rt, var("a")),
                _ => unreachable!(),
            };

            b.push([result]);
            b.build()
        };
        rt.def(op, definition);
    }

    var(op)
}

pub fn i32_clz(rt: &mut RuntimeGenerator) -> Instruction {
    unop(rt, "Clz32")
}

pub fn i64_clz(rt: &mut RuntimeGenerator) -> Instruction {
    unop(rt, "Clz64")
}

pub fn i32_ctz(rt: &mut RuntimeGenerator) -> Instruction {
    unop(rt, "Ctz32")
}

pub fn i64_ctz(rt: &mut RuntimeGenerator) -> Instruction {
    unop(rt, "Ctz64")
}

pub fn i32_popcnt(rt: &mut RuntimeGenerator) -> Instruction {
    unop(rt, "PopCnt32")
}

pub fn i64_popcnt(rt: &mut RuntimeGenerator) -> Instruction {
    unop(rt, "PopCnt64")
}

fn binop(rt: &mut RuntimeGenerator, op: &str) -> Instruction {
    if !rt.has(op) {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a", "b"]);

            let result = match op {
                "And" => math::and(rt, var("a"), var("b")),
                "Or" => math::or(rt, var("a"), var("b")),
                "Xor" => math::xor(rt, var("a"), var("b")),

                "Shl32" => math::i32_shift_left(rt, var("a"), var("b")),
                "ShrU32" => math::i32_shift_right_unsigned(rt, var("a"), var("b")),
                "ShrS32" => math::i32_shift_right_signed(rt, var("a"), var("b")),
                "Shl64" => math::i64_shift_left(rt, var("a"), var("b")),
                "ShrU64" => math::i64_shift_right_unsigned(rt, var("a"), var("b")),
                "ShrS64" => math::i64_shift_right_signed(rt, var("a"), var("b")),

                "Rotl32" => math::i32_rotate_left(rt, var("a"), var("b")),
                "Rotr32" => math::i32_rotate_right(rt, var("a"), var("b")),
                "Rotl64" => math::i64_rotate_left(rt, var("a"), var("b")),
                "Rotr64" => math::i64_rotate_right(rt, var("a"), var("b")),

                "Add" => math::add(rt, var("a"), var("b")),
                "Sub" => math::sub(rt, var("a"), var("b")),
                "Mul" => math::mul(rt, var("a"), var("b")),
                _ => unreachable!(),
            };

            b.push([result]);
            b.build()
        };
        rt.def(op, definition);
    }

    var(op)
}

pub fn and(rt: &mut RuntimeGenerator) -> Instruction {
    binop(rt, "And")
}

pub fn or(rt: &mut RuntimeGenerator) -> Instruction {
    binop(rt, "Or")
}

pub fn xor(rt: &mut RuntimeGenerator) -> Instruction {
    binop(rt, "Xor")
}

pub fn add(rt: &mut RuntimeGenerator) -> Instruction {
    binop(rt, "Add")
}

pub fn sub(rt: &mut RuntimeGenerator) -> Instruction {
    binop(rt, "Sub")
}

pub fn mul(rt: &mut RuntimeGenerator) -> Instruction {
    binop(rt, "Mul")
}

pub fn i32_shl(rt: &mut RuntimeGenerator) -> Instruction {
    binop(rt, "Shl32")
}

pub fn i32_shr_u(rt: &mut RuntimeGenerator) -> Instruction {
    binop(rt, "ShrU32")
}

pub fn i32_shr_s(rt: &mut RuntimeGenerator) -> Instruction {
    binop(rt, "ShrS32")
}

pub fn i64_shl(rt: &mut RuntimeGenerator) -> Instruction {
    binop(rt, "Shl64")
}

pub fn i64_shr_u(rt: &mut RuntimeGenerator) -> Instruction {
    binop(rt, "ShrU64")
}

pub fn i64_shr_s(rt: &mut RuntimeGenerator) -> Instruction {
    binop(rt, "ShrS64")
}

pub fn i32_rotate_left(rt: &mut RuntimeGenerator) -> Instruction {
    binop(rt, "Rotl32")
}

pub fn i32_rotate_right(rt: &mut RuntimeGenerator) -> Instruction {
    binop(rt, "Rotr32")
}

pub fn i64_rotate_left(rt: &mut RuntimeGenerator) -> Instruction {
    binop(rt, "Rotl64")
}

pub fn i64_rotate_right(rt: &mut RuntimeGenerator) -> Instruction {
    binop(rt, "Rotr64")
}

/// Invokes a binary operation that returns [optional::Optional]
/// and that must trap if the result is None.
fn binop_trapping(rt: &mut RuntimeGenerator, op: &str) -> Instruction {
    if !rt.has(op) {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a", "b"]);

            b.def(
                "result",
                match op {
                    "DivU32" => math::i32_div_unsigned(rt, var("a"), var("b")),
                    "DivS32" => math::i32_div_signed(rt, var("a"), var("b")),

                    "DivU64" => math::i64_div_unsigned(rt, var("a"), var("b")),
                    "DivS64" => math::i64_div_signed(rt, var("a"), var("b")),

                    "RemU32" => math::i32_rem_unsigned(rt, var("a"), var("b")),
                    "RemS32" => math::i32_rem_signed(rt, var("a"), var("b")),

                    "RemU64" => math::i64_rem_unsigned(rt, var("a"), var("b")),
                    "RemS64" => math::i64_rem_signed(rt, var("a"), var("b")),

                    _ => unreachable!(),
                },
            );

            b.push([optional::unwrap(var("result"))]);
            b.build_check(optional::is_some(var("result")))
        };
        rt.def(op, definition);
    }

    var(op)
}

pub fn i32_div_u(rt: &mut RuntimeGenerator) -> Instruction {
    binop_trapping(rt, "DivU32")
}

pub fn i32_div_s(rt: &mut RuntimeGenerator) -> Instruction {
    binop_trapping(rt, "DivS32")
}

pub fn i64_div_u(rt: &mut RuntimeGenerator) -> Instruction {
    binop_trapping(rt, "DivU64")
}

pub fn i64_div_s(rt: &mut RuntimeGenerator) -> Instruction {
    binop_trapping(rt, "DivS64")
}

pub fn i32_rem_u(rt: &mut RuntimeGenerator) -> Instruction {
    binop_trapping(rt, "RemU32")
}

pub fn i32_rem_s(rt: &mut RuntimeGenerator) -> Instruction {
    binop_trapping(rt, "RemS32")
}

pub fn i64_rem_u(rt: &mut RuntimeGenerator) -> Instruction {
    binop_trapping(rt, "RemU64")
}

pub fn i64_rem_s(rt: &mut RuntimeGenerator) -> Instruction {
    binop_trapping(rt, "RemS64")
}

/// `op` is "Eq", "Ne", "LtU", "LeU", "GtU", "GeU", "LtS", "LeS", "GtS", or "GeS".
/// The abbreviations mean "Equal", "Not equal", "Greater/Less equal/than (Unsigned/Signed)".
fn cmp(rt: &mut RuntimeGenerator, op: &str) -> Instruction {
    if !rt.has(op) {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a", "b"]);

            let result_bit = match op {
                "Eq" => math::equal(rt, var("a"), var("b")),
                "Ne" => math::not_equal(rt, var("a"), var("b")),
                "LtU" => math::less_unsigned(rt, var("a"), var("b")),
                "LeU" => math::less_equal_unsigned(rt, var("a"), var("b")),
                "GtU" => math::greater_unsigned(rt, var("a"), var("b")),
                "GeU" => math::greater_equal_unsigned(rt, var("a"), var("b")),
                "LtS" => math::less_signed(rt, var("a"), var("b")),
                "LeS" => math::less_equal_signed(rt, var("a"), var("b")),
                "GtS" => math::greater_signed(rt, var("a"), var("b")),
                "GeS" => math::greater_equal_signed(rt, var("a"), var("b")),
                _ => unreachable!(),
            };

            let result = select(result_bit, rt.num.i32_const(0), rt.num.i32_const(1));

            b.push([result]);

            b.build()
        };
        rt.def(op, definition);
    }

    var(op)
}

pub fn eq(rt: &mut RuntimeGenerator) -> Instruction {
    cmp(rt, "Eq")
}

pub fn ne(rt: &mut RuntimeGenerator) -> Instruction {
    cmp(rt, "Ne")
}

pub fn lt_u(rt: &mut RuntimeGenerator) -> Instruction {
    cmp(rt, "LtU")
}

pub fn le_u(rt: &mut RuntimeGenerator) -> Instruction {
    cmp(rt, "LeU")
}

pub fn gt_u(rt: &mut RuntimeGenerator) -> Instruction {
    cmp(rt, "GtU")
}

pub fn ge_u(rt: &mut RuntimeGenerator) -> Instruction {
    cmp(rt, "GeU")
}

pub fn lt_s(rt: &mut RuntimeGenerator) -> Instruction {
    cmp(rt, "LtS")
}

pub fn le_s(rt: &mut RuntimeGenerator) -> Instruction {
    cmp(rt, "LeS")
}

pub fn gt_s(rt: &mut RuntimeGenerator) -> Instruction {
    cmp(rt, "GtS")
}

pub fn ge_s(rt: &mut RuntimeGenerator) -> Instruction {
    cmp(rt, "GeS")
}

pub fn i32_wrap_i64(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("I32WrapI64") {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a"]);

            let result = math::wrap_i32(rt, var("a"));

            b.push([result]);
            b.build()
        };
        rt.def("I32WrapI64", definition);
    }

    var("I32WrapI64")
}

pub fn i64_extend_i32(rt: &mut RuntimeGenerator, signed: bool) -> Instruction {
    let sign = if signed { "S" } else { "U" };

    let name = format!("I64ExtendI32{sign}");

    if !rt.has(&name) {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a"]);

            let mut result = math::widen_i64(rt, var("a"));

            if signed {
                result = math::sign_extend(rt, result, 64, 32);
            }

            b.push([result]);
            b.build()
        };
        rt.def(&name, definition);
    }

    var(name)
}

/// Args:
/// * `target_bits`: 32 or 64
/// * `source_bits`: 8, 16, 32, or 64. Must be <= `target_bits`.
pub fn extend_s(rt: &mut RuntimeGenerator, target_bits: u8, source_bits: u8) -> Instruction {
    let name = format!("I{target_bits}Extend{source_bits}S");

    if !rt.has(&name) {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a"]);

            let result = math::sign_extend(rt, var("a"), target_bits, source_bits);

            b.push([result]);
            b.build()
        };
        rt.def(&name, definition);
    }

    var(name)
}
