use super::*;

impl UtilGenerator {
    pub fn output_and_return(&mut self) -> Instruction {
        if !self.has("Output") {
            let definition = {
                let mut b = InstructionBuilder::new();
                b.pop(["a"]);

                let byte = self.i32_to_byte(var("a"));

                b.ret();
                b.build_output(byte)
            };

            self.def("Output", definition);
        }

        var("Output")
    }

    pub fn input_and_return(&mut self) -> Instruction {
        if !self.has("Input") {
            let definition = {
                let mut b = InstructionBuilder::new();

                let invalid_input = self.num.i32_const(u32::MAX);

                let byte_to_i32 = |byte| {
                    number::make_i32([
                        number::null_byte(),
                        number::null_byte(),
                        number::null_byte(),
                        byte,
                    ])
                };

                b.push([select(
                    optional::is_some(var("inp")),
                    invalid_input,
                    byte_to_i32(optional::unwrap(var("inp"))),
                )]);

                b.ret();
                b.build_input("inp")
            };

            self.def("Input", definition);
        }

        var("Input")
    }

    pub fn exit(&mut self) -> Instruction {
        if !self.has("Exit") {
            self.def("Exit", {
                let b = InstructionBuilder::new();
                b.build_exit()
            });
        }

        var("Exit")
    }
}
