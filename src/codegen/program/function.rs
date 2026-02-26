use crate::{analyzer::*, codegen::*};

use program::runtime::{InstructionInfo, RuntimeGenerator};

pub struct LabelInfo<'a> {
    /// Stack of block opening operators (`loop`, `if`, `block`).
    /// Given a `br X` operator, the block being breaked is `blocks[blocks.len() - 1 - X]`.
    pub blocks: &'a [&'a Operator<'a>],

    /// Stack of `end` labels (for `loop`, `if`, `block`).
    pub end_labels: &'a [&'a code::Code],

    /// If the current block is `if`, then this is its `else` label.
    pub else_label: Option<code::Code>,
}

pub struct EntrypointInfo<'a> {
    pub main_id: FuncId,
    pub start_id: Option<FuncId>,
    pub data_memory_offsets: &'a [u32],
}

pub fn function(rt: &mut RuntimeGenerator, func: &Func, types: &GlobalTypeInfo) -> code::Code {
    let mut code = code::CodeBuilder::new();

    let function_end_label = code.make_label();

    code.push(rt.enter(func));

    // Ignore the last "end" operator that ends the function
    for op in &func.operators[..func.operators.len() - 1] {
        let instr = rt.instruction(&mut InstructionInfo {
            op,
            types,
            labels: LabelInfo {
                blocks: &[],
                end_labels: &[&function_end_label],
                else_label: None,
            },
        });

        code.push(instr);
    }

    code.push_label(function_end_label);

    code.push(rt.leave(func.func_type));

    code.build()
}

pub fn input_function(rt: &mut RuntimeGenerator) -> code::Code {
    let mut code = code::CodeBuilder::new();
    code.push(rt.input_and_return());
    code.build()
}

pub fn output_function(rt: &mut RuntimeGenerator) -> code::Code {
    let mut code = code::CodeBuilder::new();
    code.push(rt.output_and_return());
    code.build()
}

pub fn exit_function(rt: &mut RuntimeGenerator) -> code::Code {
    let mut code = code::CodeBuilder::new();
    code.push(rt.exit());
    code.build()
}

pub fn entrypoint(rt: &mut RuntimeGenerator, info: &EntrypointInfo) -> io_command::IoCommand {
    let mut chain = unreachable();

    let instr = rt.exit();
    chain = apply(instr, [chain]);

    for (data_id, target_offset) in info.data_memory_offsets.iter().enumerate() {
        // TODO init data segments
    }

    if let Some(start_id) = info.start_id {
        let start_id = rt.num.id_const(start_id as u16);
        let instr = rt.call(start_id);
        chain = apply(instr, [chain]);
    }

    let main_id = rt.num.id_const(info.main_id as u16);
    let instr = rt.call(main_id);
    chain = apply(instr, [chain]);

    chain
}
