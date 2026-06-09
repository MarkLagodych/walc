use super::*;

pub fn set_trap_handler(rt: &mut RuntimeGenerator, handle_trap: Option<code::Code>) -> Instruction {
    if !rt.has("InitTrap") {
        let trap_handler_global_id = rt.num.id_const(global_ids::TRAP_HANDLER_GLOBAL_ID);

        let definition = abs(["handle_trap"], {
            let mut b = InstructionContextBuilder::new();

            b.set_global(trap_handler_global_id, var("handle_trap"));

            b.build_simple_instruction()
        });

        rt.def("InitTrap", definition);
    }

    let handle_trap = match handle_trap {
        Some(code) => optional::some(code),
        None => optional::none(),
    };

    apply(var("InitTrap"), [handle_trap])
}

#[repr(u32)]
pub enum TrapCode {
    ReachedUnreachable = 0,
    DivisionError = 1,
    UsedUnsupportedFloatArithmetic = 2,
}

pub fn trap(rt: &mut RuntimeGenerator, trap_code: TrapCode) -> Instruction {
    if !rt.has("Trap") {
        let trap_handler_global_id = rt.num.id_const(global_ids::TRAP_HANDLER_GLOBAL_ID);

        let definition = abs(["trap_code"], {
            let mut b = InstructionContextBuilder::new();

            b.get_global("handler", trap_handler_global_id);

            // Drop the old state (except the memory and the globals)
            b.set_stack(data_stack::empty());
            b.set_locals(locals::new());
            b.set_trace(stack::empty());

            b.push([var("trap_code")]);

            // Exit the program after the handler returns
            b.push_trace(code::single(io::exit(rt)));

            let cmd: io_command::IoCommand = select(
                optional::is_some(var("handler")),
                io_command::exit(),
                exec(optional::unwrap(var("handler"))),
            );

            instr(b.build(cmd))
        });

        rt.def("Trap", definition);
    }

    apply(var("Trap"), [rt.num.i32_const(trap_code as u32)])
}
