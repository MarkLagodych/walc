use super::*;

pub fn local_get(rt: &mut RuntimeGenerator, local_index: u32) -> Instruction {
    if !rt.has("LGet") {
        rt.def("LGet", {
            abs(["id"], {
                let mut b = InstructionBuilder::new();
                b.get_local("a", var("id"));
                b.push([var("a")]);
                b.build()
            })
        });
    }

    let id = rt.num.id_const(local_index as u16);
    apply(var("LGet"), [id])
}

pub fn local_set(rt: &mut RuntimeGenerator, local_index: u32) -> Instruction {
    if !rt.has("LSet") {
        rt.def("LSet", {
            abs(["id"], {
                let mut b = InstructionBuilder::new();
                b.pop(["a"]);
                b.set_local(var("id"), var("a"));
                b.build()
            })
        });
    }

    let id = rt.num.id_const(local_index as u16);
    apply(var("LSet"), [id])
}

pub fn local_tee(rt: &mut RuntimeGenerator, local_index: u32) -> Instruction {
    if !rt.has("LTee") {
        rt.def("LTee", {
            abs(["id"], {
                let mut b = InstructionBuilder::new();
                b.get_top("a");
                b.set_local(var("id"), var("a"));
                b.build()
            })
        });
    }

    let id = rt.num.id_const(local_index as u16);
    apply(var("LTee"), [id])
}

pub fn global_get(rt: &mut RuntimeGenerator, global_index: u32) -> Instruction {
    if !rt.has("GGet") {
        rt.def("GGet", {
            abs(["id"], {
                let mut b = InstructionBuilder::new();
                b.get_global("a", var("id"));
                b.push([var("a")]);
                b.build()
            })
        });
    }

    let id = rt.num.id_const(global_index as u16);
    apply(var("GGet"), [id])
}

pub fn global_set(rt: &mut RuntimeGenerator, global_index: u32) -> Instruction {
    if !rt.has("GSet") {
        rt.def("GSet", {
            abs(["id"], {
                let mut b = InstructionBuilder::new();
                b.pop(["a"]);
                b.set_global(var("id"), var("a"));
                b.build()
            })
        });
    }

    let id = rt.num.id_const(global_index as u16);
    apply(var("GSet"), [id])
}
