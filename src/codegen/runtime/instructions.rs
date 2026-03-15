pub mod control_flow;
pub mod io;
pub mod memory;
pub mod numeric;
pub mod parametric;
pub mod variable;

use super::*;

use crate::codegen::{
    function::BlockStack,
    instruction::{Instruction, InstructionBuilder},
};

use crate::analyzer::{Func, Operator};

pub fn instruction(rt: &mut RuntimeGenerator, op: &Operator, blocks: &BlockStack) -> Instruction {
    use Operator::*;

    match op {
        // ==================================================================================
        // Parametric instructions
        Drop => parametric::drop(rt),
        Select => parametric::select(rt),

        // ==================================================================================
        // Constrol flow instructions
        Nop => instruction::nop(),
        Unreachable => io::exit(rt),

        Call { function_index } => control_flow::call(rt, *function_index),
        CallIndirect { .. } => control_flow::call_indirect(rt),

        Loop { .. } | If { .. } | Block { .. } => control_flow::begin_block(rt, blocks),
        Else => control_flow::block_else(rt, blocks),
        End => control_flow::end_block(rt, blocks),

        Br { relative_depth } => control_flow::br(rt, blocks, *relative_depth),
        BrIf { relative_depth } => control_flow::br_if(rt, blocks, *relative_depth),
        BrTable { targets } => control_flow::br_table(rt, blocks, targets),
        Return => control_flow::ret(rt, blocks),

        // ==================================================================================
        // Variable instructions
        LocalGet { local_index } => variable::local_get(rt, *local_index),
        LocalSet { local_index } => variable::local_set(rt, *local_index),
        LocalTee { local_index } => variable::local_tee(rt, *local_index),
        GlobalGet { global_index } => variable::global_get(rt, *global_index),
        GlobalSet { global_index } => variable::global_set(rt, *global_index),

        // ==================================================================================
        // Memory instructions
        MemorySize { .. } => memory::size(rt),
        MemoryGrow { .. } => memory::grow(rt),

        MemoryFill { .. } => memory::fill(rt),
        MemoryCopy { .. } => memory::copy(rt),

        I32Load { memarg, .. } => memory::load(rt, memarg.offset as u32, 32, 32, false),
        I64Load { memarg, .. } => memory::load(rt, memarg.offset as u32, 64, 64, false),

        I32Load8U { memarg, .. } => memory::load(rt, memarg.offset as u32, 32, 8, false),
        I32Load8S { memarg, .. } => memory::load(rt, memarg.offset as u32, 32, 8, true),

        I32Load16U { memarg, .. } => memory::load(rt, memarg.offset as u32, 32, 16, false),
        I32Load16S { memarg, .. } => memory::load(rt, memarg.offset as u32, 32, 16, true),

        I64Load8U { memarg, .. } => memory::load(rt, memarg.offset as u32, 64, 8, false),
        I64Load8S { memarg, .. } => memory::load(rt, memarg.offset as u32, 64, 8, true),

        I64Load16U { memarg, .. } => memory::load(rt, memarg.offset as u32, 64, 16, false),
        I64Load16S { memarg, .. } => memory::load(rt, memarg.offset as u32, 64, 16, true),

        I64Load32U { memarg, .. } => memory::load(rt, memarg.offset as u32, 64, 32, false),
        I64Load32S { memarg, .. } => memory::load(rt, memarg.offset as u32, 64, 32, true),

        I32Store { memarg, .. } => memory::store(rt, memarg.offset as u32, 32, 32),
        I64Store { memarg, .. } => memory::store(rt, memarg.offset as u32, 64, 64),

        I32Store8 { memarg, .. } => memory::store(rt, memarg.offset as u32, 32, 8),
        I32Store16 { memarg, .. } => memory::store(rt, memarg.offset as u32, 32, 16),

        I64Store8 { memarg, .. } => memory::store(rt, memarg.offset as u32, 64, 8),
        I64Store16 { memarg, .. } => memory::store(rt, memarg.offset as u32, 64, 16),
        I64Store32 { memarg, .. } => memory::store(rt, memarg.offset as u32, 64, 32),

        // ==================================================================================
        // Integer instructions
        I32Const { .. } | I64Const { .. } => numeric::push_const(rt, op),

        I32WrapI64 => numeric::i32_wrap_i64(rt),

        I64ExtendI32U => numeric::i64_extend_i32(rt, false),
        I64ExtendI32S => numeric::i64_extend_i32(rt, true),

        I32Extend8S => numeric::extend_s(rt, 32, 8),
        I32Extend16S => numeric::extend_s(rt, 32, 16),
        I64Extend8S => numeric::extend_s(rt, 64, 8),
        I64Extend16S => numeric::extend_s(rt, 64, 16),
        I64Extend32S => numeric::extend_s(rt, 64, 32),

        I32Eqz | I64Eqz => numeric::eqz(rt),
        I32Eq | I64Eq => numeric::eq(rt),
        I32Ne | I64Ne => numeric::ne(rt),

        I32And | I64And => numeric::and(rt),
        I32Or | I64Or => numeric::or(rt),
        I32Xor | I64Xor => numeric::xor(rt),

        I32Shl => numeric::i32_shl(rt),
        I32ShrU => numeric::i32_shr_u(rt),
        I32ShrS => numeric::i32_shr_s(rt),

        I64Shl => numeric::i64_shl(rt),
        I64ShrU => numeric::i64_shr_u(rt),
        I64ShrS => numeric::i64_shr_s(rt),

        I32Rotl => numeric::i32_rotate_left(rt),
        I32Rotr => numeric::i32_rotate_right(rt),

        I64Rotl => numeric::i64_rotate_left(rt),
        I64Rotr => numeric::i64_rotate_right(rt),

        I32Clz => numeric::i32_clz(rt),
        I32Ctz => numeric::i32_ctz(rt),
        I32Popcnt => numeric::i32_popcnt(rt),

        I64Clz => numeric::i64_clz(rt),
        I64Ctz => numeric::i64_ctz(rt),
        I64Popcnt => numeric::i64_popcnt(rt),

        I32LtU | I64LtU => numeric::lt_u(rt),
        I32LtS | I64LtS => numeric::lt_s(rt),

        I32LeU | I64LeU => numeric::le_u(rt),
        I32LeS | I64LeS => numeric::le_s(rt),

        I32GtU | I64GtU => numeric::gt_u(rt),
        I32GtS | I64GtS => numeric::gt_s(rt),

        I32GeU | I64GeU => numeric::ge_u(rt),
        I32GeS | I64GeS => numeric::ge_s(rt),

        I32Add | I64Add => numeric::add(rt),
        I32Sub | I64Sub => numeric::sub(rt),
        I32Mul | I64Mul => numeric::mul(rt),

        I32DivU => numeric::i32_div_u(rt),
        I32DivS => numeric::i32_div_s(rt),

        I64DivU => numeric::i64_div_u(rt),
        I64DivS => numeric::i64_div_s(rt),

        I32RemU => numeric::i32_rem_u(rt),
        I32RemS => numeric::i32_rem_s(rt),

        I64RemU => numeric::i64_rem_u(rt),
        I64RemS => numeric::i64_rem_s(rt),

        // ==================================================================================
        // Floating-point instructions

        // Floats are not supported, but to avoid as much compilation problems as possible,
        // all floats are reinterpreted as integers and any floating-point operations are
        // replaced with traps.
        F32Const { .. } | F64Const { .. } => numeric::push_const(rt, op),

        F32Load { memarg, .. } => memory::load(rt, memarg.offset as u32, 32, 32, false),
        F64Load { memarg, .. } => memory::load(rt, memarg.offset as u32, 64, 64, false),

        F32Store { memarg, .. } => memory::store(rt, memarg.offset as u32, 32, 32),
        F64Store { memarg, .. } => memory::store(rt, memarg.offset as u32, 64, 64),

        F32ReinterpretI32 => instruction::nop(),
        F64ReinterpretI64 => instruction::nop(),
        I32ReinterpretF32 => instruction::nop(),
        I64ReinterpretF64 => instruction::nop(),

        F32Abs | F64Abs | F32Neg | F64Neg | F32Ceil | F64Ceil | F32Floor | F64Floor | F32Trunc
        | F64Trunc | F32Nearest | F64Nearest | F32Sqrt | F64Sqrt | F32Add | F64Add | F32Sub
        | F64Sub | F32Mul | F64Mul | F32Div | F64Div | F32Min | F64Min | F32Max | F64Max
        | F32Copysign | F64Copysign | F32Eq | F64Eq | F32Ne | F64Ne | F32Lt | F64Lt | F32Le
        | F64Le | F32Gt | F64Gt | F32Ge | F64Ge | I32TruncF32S | I32TruncF32U | I32TruncF64S
        | I32TruncF64U | I64TruncF32S | I64TruncF32U | I64TruncF64S | I64TruncF64U
        | F32ConvertI32S | F32ConvertI32U | F32ConvertI64S | F32ConvertI64U | F64ConvertI32S
        | F64ConvertI32U | F64ConvertI64S | F64ConvertI64U | F32DemoteF64 | F64PromoteF32 => {
            eprintln!("Warning: floating-point arithmetic unsupported, replaced with traps");

            io::exit(rt)
        }

        _ => unreachable!("unsupported instruction: {op:?}"),
    }
}

pub fn func_prologue(rt: &mut RuntimeGenerator, func: &Func) -> Instruction {
    let mut b = InstructionBuilder::new();

    let param_count = func.func_type.params().len();

    b.pop((0..param_count).map(|i| format!("p{i:x}")));

    let mut locals = Vec::new();
    locals.extend((0..param_count).map(|i| var(format!("p{i:x}"))));
    locals.extend(func.local_types.iter().map(|ty| rt.num.default_const(*ty)));

    b.push_locals_frame(table::from(locals));
    b.push_stack_frame();

    b.build()
}
