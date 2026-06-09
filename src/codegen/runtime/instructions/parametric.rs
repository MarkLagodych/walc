use super::*;

pub fn drop(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("Drop") {
        rt.def("Drop", {
            let mut b = InstructionContextBuilder::new();
            b.pop(["a"]);
            b.build_simple_instruction()
        });
    }

    var("Drop")
}

pub fn select(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("Select") {
        let definition = {
            let mut b = InstructionContextBuilder::new();

            b.pop(["val_if_not_zero", "val_if_zero", "x"]);

            b.push([core::select(
                math::is_zero(rt, var("x")),
                var("val_if_not_zero"),
                var("val_if_zero"),
            )]);

            b.build_simple_instruction()
        };

        rt.def("Select", definition);
    }

    var("Select")
}
