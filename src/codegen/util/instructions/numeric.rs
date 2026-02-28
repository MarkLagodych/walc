use super::*;

impl UtilGenerator {
    pub fn push_const(&mut self, op: &Operator) -> Instruction {
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
                b.push([self.num_is_zero(var("a"))]);
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
                b.push([self.num_equal(var("a"), var("b"))]);
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
                b.push([self.num_not_equal(var("a"), var("b"))]);
                b.build()
            };
            self.def("Ne", definition);
        }

        var("Ne")
    }
}
