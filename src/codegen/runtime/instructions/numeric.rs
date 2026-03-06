use super::*;

pub fn i_const(rt: &mut RuntimeGenerator, op: &Operator) -> Instruction {
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

pub fn i_eqz(rt: &mut RuntimeGenerator) -> Instruction {
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

pub fn i_eq(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("Eq") {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a", "b"]);

            let result = select(
                math::equal(rt, var("a"), var("b")),
                rt.num.i32_const(0),
                rt.num.i32_const(1),
            );

            b.push([result]);

            b.build()
        };
        rt.def("Eq", definition);
    }

    var("Eq")
}

pub fn i_ne(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("Ne") {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a", "b"]);

            let result = select(
                math::not_equal(rt, var("a"), var("b")),
                rt.num.i32_const(0),
                rt.num.i32_const(1),
            );

            b.push([result]);

            b.build()
        };
        rt.def("Ne", definition);
    }

    var("Ne")
}

pub fn i_and(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("And") {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a", "b"]);
            b.push([math::and(rt, var("a"), var("b"))]);
            b.build()
        };
        rt.def("And", definition);
    }

    var("And")
}

pub fn i_or(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("Or") {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a", "b"]);
            b.push([math::or(rt, var("a"), var("b"))]);
            b.build()
        };
        rt.def("Or", definition);
    }

    var("Or")
}

pub fn i_xor(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("Xor") {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a", "b"]);
            b.push([math::xor(rt, var("a"), var("b"))]);
            b.build()
        };
        rt.def("Xor", definition);
    }

    var("Xor")
}

pub fn i_lt_u(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("Lt") {
        let definition = {
            let mut b = InstructionBuilder::new();

            b.pop(["a", "b"]);

            let result = select(
                math::less_unsigned(rt, var("a"), var("b")),
                rt.num.i32_const(0),
                rt.num.i32_const(1),
            );

            b.push([result]);

            b.build()
        };
        rt.def("Lt", definition);
    }

    var("Lt")
}

pub fn i_le_u(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("Le") {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a", "b"]);

            let result = select(
                math::less_equal_unsigned(rt, var("a"), var("b")),
                rt.num.i32_const(0),
                rt.num.i32_const(1),
            );

            b.push([result]);

            b.build()
        };
        rt.def("Le", definition);
    }

    var("Le")
}

pub fn i_gt_u(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("Gt") {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a", "b"]);

            let result = select(
                math::greater_unsigned(rt, var("a"), var("b")),
                rt.num.i32_const(0),
                rt.num.i32_const(1),
            );

            b.push([result]);

            b.build()
        };
        rt.def("Gt", definition);
    }

    var("Gt")
}

pub fn i_ge_u(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("Ge") {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a", "b"]);

            let result = select(
                math::greater_equal_unsigned(rt, var("a"), var("b")),
                rt.num.i32_const(0),
                rt.num.i32_const(1),
            );

            b.push([result]);

            b.build()
        };
        rt.def("Ge", definition);
    }

    var("Ge")
}

pub fn i_lt_s(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("LtS") {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a", "b"]);

            let result = select(
                math::less_signed(rt, var("a"), var("b")),
                rt.num.i32_const(0),
                rt.num.i32_const(1),
            );

            b.push([result]);

            b.build()
        };
        rt.def("LtS", definition);
    }

    var("LtS")
}

pub fn i_le_s(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("LeS") {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a", "b"]);

            let result = select(
                math::less_equal_signed(rt, var("a"), var("b")),
                rt.num.i32_const(0),
                rt.num.i32_const(1),
            );

            b.push([result]);
            b.build()
        };
        rt.def("LeS", definition);
    }

    var("LeS")
}

pub fn i_gt_s(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("GtS") {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a", "b"]);

            let result = select(
                math::greater_signed(rt, var("a"), var("b")),
                rt.num.i32_const(0),
                rt.num.i32_const(1),
            );

            b.push([result]);
            b.build()
        };
        rt.def("GtS", definition);
    }

    var("GtS")
}

pub fn i_ge_s(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("GeS") {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a", "b"]);

            let result = select(
                math::greater_equal_signed(rt, var("a"), var("b")),
                rt.num.i32_const(0),
                rt.num.i32_const(1),
            );

            b.push([result]);
            b.build()
        };
        rt.def("GeS", definition);
    }

    var("GeS")
}

pub fn i_add(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("Add") {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a", "b"]);
            b.push([math::add(rt, var("a"), var("b"))]);
            b.build()
        };
        rt.def("Add", definition);
    }

    var("Add")
}

pub fn i_sub(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("Sub") {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a", "b"]);
            b.push([math::sub(rt, var("a"), var("b"))]);
            b.build()
        };
        rt.def("Sub", definition);
    }

    var("Sub")
}

pub fn i32_wrap_i64(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("I32WrapI64") {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a"]);

            let result = math::i64_to_i32(rt, var("a"));

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

            let mut result = math::i32_to_i64(rt, var("a"));

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
pub fn i_extend_s(rt: &mut RuntimeGenerator, target_bits: u8, source_bits: u8) -> Instruction {
    let name = format!("I{target_bits}Extend{source_bits}S");

    if !rt.has(&name) {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a"]);

            let result = math::sign_extend(rt, var("a"), 64, 32);

            b.push([result]);
            b.build()
        };
        rt.def(&name, definition);
    }

    var(name)
}
