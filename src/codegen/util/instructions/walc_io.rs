use super::*;

impl UtilGenerator {
    pub fn output_and_return(&mut self) -> Instruction {
        if !self.has("Output") {
            self.def("Output", {
                let mut b = InstructionBuilder::new();
                b.pop(["a"]);
                // TODO convert a to byte
                let byte = number::reverse_bits(var("a"));
                b.ret();
                b.build_output(byte)
            });
        }

        var("Output")
    }

    pub fn input_and_return(&mut self) -> Instruction {
        if !self.has("Input") {
            let invalid_input = self.num.i32_const(u32::MAX);

            self.def("Input", {
                let mut b = InstructionBuilder::new();
                b.push([select(
                    optional::is_some(var("inp")),
                    invalid_input,
                    // TODO convert input to i32
                    optional::unwrap(var("inp")),
                )]);
                b.ret();
                b.build_input("inp")
            });
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
