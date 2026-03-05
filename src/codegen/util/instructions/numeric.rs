use super::*;

impl UtilGenerator {
    pub fn const_push(&mut self, op: &Operator) -> Instruction {
        if !self.has("Push") {
            self.def("Push", {
                abs(["item"], {
                    let mut b = InstructionBuilder::new();
                    b.push([var("item")]);
                    b.build()
                })
            });
        }

        let item = self.num.with_init_value(op);
        apply(var("Push"), [item])
    }

    pub fn eqz(&mut self) -> Instruction {
        if !self.has("Eqz") {
            let definition = {
                let mut b = InstructionBuilder::new();
                b.pop(["a"]);

                let result = select(
                    self.num_is_zero(var("a")),
                    self.num.i32_const(0),
                    self.num.i32_const(1),
                );

                b.push([result]);

                b.build()
            };
            self.def("Eqz", definition);
        }

        var("Eqz")
    }

    pub fn eq(&mut self) -> Instruction {
        if !self.has("Eq") {
            let definition = {
                let mut b = InstructionBuilder::new();
                b.pop(["a", "b"]);

                let result = select(
                    self.num_equal(var("a"), var("b")),
                    self.num.i32_const(0),
                    self.num.i32_const(1),
                );

                b.push([result]);

                b.build()
            };
            self.def("Eq", definition);
        }

        var("Eq")
    }

    pub fn ne(&mut self) -> Instruction {
        if !self.has("Ne") {
            let definition = {
                let mut b = InstructionBuilder::new();
                b.pop(["a", "b"]);

                let result = select(
                    self.num_not_equal(var("a"), var("b")),
                    self.num.i32_const(0),
                    self.num.i32_const(1),
                );

                b.push([result]);

                b.build()
            };
            self.def("Ne", definition);
        }

        var("Ne")
    }

    pub fn and(&mut self) -> Instruction {
        if !self.has("And") {
            let definition = {
                let mut b = InstructionBuilder::new();
                b.pop(["a", "b"]);
                b.push([self.num_and(var("a"), var("b"))]);
                b.build()
            };
            self.def("And", definition);
        }

        var("And")
    }

    pub fn or(&mut self) -> Instruction {
        if !self.has("Or") {
            let definition = {
                let mut b = InstructionBuilder::new();
                b.pop(["a", "b"]);
                b.push([self.num_or(var("a"), var("b"))]);
                b.build()
            };
            self.def("Or", definition);
        }

        var("Or")
    }

    pub fn xor(&mut self) -> Instruction {
        if !self.has("Xor") {
            let definition = {
                let mut b = InstructionBuilder::new();
                b.pop(["a", "b"]);
                b.push([self.num_xor(var("a"), var("b"))]);
                b.build()
            };
            self.def("Xor", definition);
        }

        var("Xor")
    }

    pub fn lt_u(&mut self) -> Instruction {
        if !self.has("Lt") {
            let definition = {
                let mut b = InstructionBuilder::new();

                b.pop(["a", "b"]);

                let result = select(
                    self.num_less_unsigned(var("a"), var("b")),
                    self.num.i32_const(0),
                    self.num.i32_const(1),
                );

                b.push([result]);

                b.build()
            };
            self.def("Lt", definition);
        }

        var("Lt")
    }

    pub fn le_u(&mut self) -> Instruction {
        if !self.has("Le") {
            let definition = {
                let mut b = InstructionBuilder::new();
                b.pop(["a", "b"]);

                let result = select(
                    self.num_less_equal_unsigned(var("a"), var("b")),
                    self.num.i32_const(0),
                    self.num.i32_const(1),
                );

                b.push([result]);

                b.build()
            };
            self.def("Le", definition);
        }

        var("Le")
    }

    pub fn gt_u(&mut self) -> Instruction {
        if !self.has("Gt") {
            let definition = {
                let mut b = InstructionBuilder::new();
                b.pop(["a", "b"]);

                let result = select(
                    self.num_greater_unsigned(var("a"), var("b")),
                    self.num.i32_const(0),
                    self.num.i32_const(1),
                );

                b.push([result]);

                b.build()
            };
            self.def("Gt", definition);
        }

        var("Gt")
    }

    pub fn ge_u(&mut self) -> Instruction {
        if !self.has("Ge") {
            let definition = {
                let mut b = InstructionBuilder::new();
                b.pop(["a", "b"]);

                let result = select(
                    self.num_greater_equal_unsigned(var("a"), var("b")),
                    self.num.i32_const(0),
                    self.num.i32_const(1),
                );

                b.push([result]);

                b.build()
            };
            self.def("Ge", definition);
        }

        var("Ge")
    }

    pub fn lt_s(&mut self) -> Instruction {
        if !self.has("LtS") {
            let definition = {
                let mut b = InstructionBuilder::new();
                b.pop(["a", "b"]);

                let result = select(
                    self.num_less_signed(var("a"), var("b")),
                    self.num.i32_const(0),
                    self.num.i32_const(1),
                );

                b.push([result]);

                b.build()
            };
            self.def("LtS", definition);
        }

        var("LtS")
    }

    pub fn le_s(&mut self) -> Instruction {
        if !self.has("LeS") {
            let definition = {
                let mut b = InstructionBuilder::new();
                b.pop(["a", "b"]);

                let result = select(
                    self.num_less_equal_signed(var("a"), var("b")),
                    self.num.i32_const(0),
                    self.num.i32_const(1),
                );

                b.push([result]);
                b.build()
            };
            self.def("LeS", definition);
        }

        var("LeS")
    }

    pub fn gt_s(&mut self) -> Instruction {
        if !self.has("GtS") {
            let definition = {
                let mut b = InstructionBuilder::new();
                b.pop(["a", "b"]);

                let result = select(
                    self.num_greater_signed(var("a"), var("b")),
                    self.num.i32_const(0),
                    self.num.i32_const(1),
                );

                b.push([result]);
                b.build()
            };
            self.def("GtS", definition);
        }

        var("GtS")
    }

    pub fn ge_s(&mut self) -> Instruction {
        if !self.has("GeS") {
            let definition = {
                let mut b = InstructionBuilder::new();
                b.pop(["a", "b"]);

                let result = select(
                    self.num_greater_equal_signed(var("a"), var("b")),
                    self.num.i32_const(0),
                    self.num.i32_const(1),
                );

                b.push([result]);
                b.build()
            };
            self.def("GeS", definition);
        }

        var("GeS")
    }

    pub fn add(&mut self) -> Instruction {
        if !self.has("Add") {
            let definition = {
                let mut b = InstructionBuilder::new();
                b.pop(["a", "b"]);
                b.push([self.num_add(var("a"), var("b"))]);
                b.build()
            };
            self.def("Add", definition);
        }

        var("Add")
    }

    pub fn sub(&mut self) -> Instruction {
        if !self.has("Sub") {
            let definition = {
                let mut b = InstructionBuilder::new();
                b.pop(["a", "b"]);
                b.push([self.num_sub(var("a"), var("b"))]);
                b.build()
            };
            self.def("Sub", definition);
        }

        var("Sub")
    }

    pub fn i32_wrap_i64(&mut self) -> Instruction {
        if !self.has("I32WrapI64") {
            let definition = {
                let mut b = InstructionBuilder::new();
                b.pop(["a"]);

                let result = self.i64_to_i32(var("a"));

                b.push([result]);
                b.build()
            };
            self.def("I32WrapI64", definition);
        }

        var("I32WrapI64")
    }

    pub fn i64_extend_i32(&mut self, signed: bool) -> Instruction {
        let sign = if signed { "S" } else { "U" };

        let name = format!("I64ExtendI32{sign}");

        if !self.has(&name) {
            let definition = {
                let mut b = InstructionBuilder::new();
                b.pop(["a"]);

                let mut result = self.i32_to_i64(var("a"));

                if signed {
                    result = self.num_sign_extend(result, 64, 32);
                }

                b.push([result]);
                b.build()
            };
            self.def(&name, definition);
        }

        var(name)
    }

    /// Args:
    /// * `target_bits`: 32 or 64
    /// * `source_bits`: 8, 16, 32, or 64. Must be <= `target_bits`.
    pub fn i_extend_s(&mut self, target_bits: u8, source_bits: u8) -> Instruction {
        let name = format!("I{target_bits}Extend{source_bits}S");

        if !self.has(&name) {
            let definition = {
                let mut b = InstructionBuilder::new();
                b.pop(["a"]);

                let result = self.num_sign_extend(var("a"), target_bits, source_bits);

                b.push([result]);
                b.build()
            };
            self.def(&name, definition);
        }

        var(name)
    }
}
