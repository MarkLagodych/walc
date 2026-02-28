use super::*;

impl UtilGenerator {
    pub fn local_get(&mut self, local_index: u32) -> Instruction {
        if !self.has("LGet") {
            self.def("LGet", {
                abs(["id"], {
                    let mut b = InstructionBuilder::new();
                    b.get_local("a", var("id"));
                    b.push([var("a")]);
                    b.build()
                })
            });
        }

        let id = self.num.id_const(local_index as u16);
        apply(var("LGet"), [id])
    }

    pub fn local_set(&mut self, local_index: u32) -> Instruction {
        if !self.has("LSet") {
            self.def("LSet", {
                abs(["id"], {
                    let mut b = InstructionBuilder::new();
                    b.pop(["a"]);
                    b.set_local(var("id"), var("a"));
                    b.build()
                })
            });
        }

        let id = self.num.id_const(local_index as u16);
        apply(var("LSet"), [id])
    }

    pub fn local_tee(&mut self, local_index: u32) -> Instruction {
        if !self.has("LTee") {
            self.def("LTee", {
                abs(["id"], {
                    let mut b = InstructionBuilder::new();
                    b.get_top("a");
                    b.set_local(var("id"), var("a"));
                    b.build()
                })
            });
        }

        let id = self.num.id_const(local_index as u16);
        apply(var("LTee"), [id])
    }

    pub fn global_get(&mut self, global_index: u32) -> Instruction {
        if !self.has("GGet") {
            self.def("GGet", {
                abs(["id"], {
                    let mut b = InstructionBuilder::new();
                    b.get_global("a", var("id"));
                    b.push([var("a")]);
                    b.build()
                })
            });
        }

        let id = self.num.id_const(global_index as u16);
        apply(var("GGet"), [id])
    }

    pub fn global_set(&mut self, global_index: u32) -> Instruction {
        if !self.has("GSet") {
            self.def("GSet", {
                abs(["id"], {
                    let mut b = InstructionBuilder::new();
                    b.pop(["a"]);
                    b.set_global(var("id"), var("a"));
                    b.build()
                })
            });
        }

        let id = self.num.id_const(global_index as u16);
        apply(var("GSet"), [id])
    }
}
