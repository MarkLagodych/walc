use super::*;

pub fn drop(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("Drop") {
        rt.def("Drop", {
            let mut b = InstructionBuilder::new();
            b.pop(["a"]);
            b.build()
        });
    }

    var("Drop")
}

pub fn select(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("Select") {
        let definition = {
            let mut b = InstructionBuilder::new();

            b.pop(["val_if_not_zero", "val_if_zero", "x"]);

            b.push([core::select(
                math::is_zero(rt, var("x")),
                var("val_if_not_zero"),
                var("val_if_zero"),
            )]);

            b.build()
        };

        rt.def("Select", definition);
    }

    var("Select")
}
