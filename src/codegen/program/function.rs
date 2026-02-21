mod instruction;
pub use instruction::InstructionDefinitionBuilder;

mod labels;
use labels::*;

use crate::{analyzer::*, codegen::*};

/// Made of instructions (see [`instruction::Instruction`]) similarly to (linked) lists
/// (see [`list::List`]), so e.g. `(Instr1 (Instr2 (Instr3 unreachable)))` is a typical chain of
/// simple (non-control) instructions.
///
/// However, instruction chains get more complicated when it comes to control instructions.
/// For them we need special "labels" that point to specific segments of the chain, so that
/// control instructions can "jump" to them.
///
/// This is done by constructing the chain in parts: each subchain is assigned to a variable
/// (whose name is practically a label) and is used as a tail for the next subchain.
///
/// For example, consider the following instructions:
/// ```wat
/// block
///     i32.eqz
///     br_if 0     ;; refers to label 1
///     call $foo
/// end             ;; label 1
/// call $bar
/// ;;(unreachable) ;; label 0
/// ```
/// Note that labels are indexed relatively and refer to block nesting depth rather than
/// concrete labels.
///
/// The corresponding instruction chain will look like this:
/// ```text
/// let label0 = unreachable in
/// let label1 = (end (call<bar> label0)) in
/// (block (i32_eqz (br_if<label1> (call<foo> label1))))
/// ```
///
/// The resulting `br_if` instruction will jump either to the next instruction (i.e. `call<foo>`)
/// or to `label1`.
pub type InstructionChain = Expr;

pub struct EntrypointInfo<'a> {
    pub main_id: FuncId,
    pub start_id: Option<FuncId>,
    pub data_memory_offsets: &'a [u32],
}

pub fn handle_function(
    func: &Func,
    types: &GlobalTypeInfo,
    consts: &mut number::ConstantDefinitionBuilder,
    instrs: &mut InstructionDefinitionBuilder,
) -> InstructionChain {
    let mut label_builder = LabelDefinitionBuilder::new(func.operators);

    let mut ops = func.operators.iter().rev();

    let function_end = ops.next().unwrap();

    let mut chain = unreachable();

    let instr = instrs.leave(func);
    chain = apply(instr, [chain]);
    chain = label_builder.insert_label_if_needed(chain, function_end);

    for op in ops {
        let instr = instrs.instruction(&mut instruction::InstructionBuildInfo {
            op,
            types,
            consts,
            labels: label_builder.get_label_info(),
        });

        chain = apply(instr, [chain]);
        chain = label_builder.insert_label_if_needed(chain, op);
    }

    let instr = instrs.enter(func, consts);
    chain = apply(instr, [chain]);

    chain = label_builder.build(chain);

    chain
}

pub fn handle_input_function(instrs: &mut InstructionDefinitionBuilder) -> InstructionChain {
    apply(instrs.input_and_return(), [unreachable()])
}

pub fn handle_output_function(instrs: &mut InstructionDefinitionBuilder) -> InstructionChain {
    apply(instrs.output_and_return(), [unreachable()])
}

pub fn handle_exit_function(instrs: &mut InstructionDefinitionBuilder) -> InstructionChain {
    apply(instrs.exit(), [unreachable()])
}

pub fn entrypoint(
    info: &EntrypointInfo,
    consts: &mut number::ConstantDefinitionBuilder,
    instrs: &mut InstructionDefinitionBuilder,
) -> io_command::IoCommand {
    let mut chain = unreachable();

    let instr = instrs.exit();
    chain = apply(instr, [chain]);

    for (data_id, target_offset) in info.data_memory_offsets.iter().enumerate() {
        // TODO init data segments
    }

    if let Some(start_id) = info.start_id {
        let instr = instrs.call(consts.id_const(start_id as u16));
        chain = apply(instr, [chain]);
    }

    let instr = instrs.call(consts.id_const(info.main_id as u16));
    chain = apply(instr, [chain]);

    instruction::start(chain)
}
