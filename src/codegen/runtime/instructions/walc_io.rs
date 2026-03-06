use super::*;

pub fn output_and_return(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("Output") {
        let definition = {
            let mut b = InstructionBuilder::new();
            b.pop(["a"]);

            let byte = math::i32_to_byte(rt, var("a"));

            b.ret();
            b.build_output(byte)
        };

        rt.def("Output", definition);
    }

    var("Output")
}

pub fn input_and_return(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("Input") {
        let definition = {
            let mut b = InstructionBuilder::new();

            let invalid_input = rt.num.i32_const(u32::MAX);

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

        rt.def("Input", definition);
    }

    var("Input")
}

pub fn exit(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("Exit") {
        rt.def("Exit", {
            let b = InstructionBuilder::new();
            b.build_exit()
        });
    }

    var("Exit")
}
