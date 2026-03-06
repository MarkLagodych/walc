use super::*;

use crate::codegen::function::{BlockLabels, BlockStack};

use crate::analyzer::{BrTable, FuncId};

pub fn call(rt: &mut RuntimeGenerator, function_id: FuncId) -> Instruction {
    if !rt.has("Call") {
        rt.def("Call", {
            abs(["funcid"], {
                let mut b = InstructionBuilder::new();
                b.call(var("funcid"));
                b.build()
            })
        });
    }

    let id = rt.num.id_const(function_id as u16);
    apply(var("Call"), [id])
}

pub fn call_indirect(rt: &mut RuntimeGenerator) -> Instruction {
    if !rt.has("CallIndirect") {
        let body = {
            let mut b = InstructionBuilder::new();
            b.pop(["a"]);

            b.def("a", math::i32_to_id(rt, var("a")));
            b.call_indirect(var("a"));

            b.build()
        };

        rt.def("CallIndirect", body);
    }

    var("CallIndirect")
}

pub fn begin_block(rt: &mut RuntimeGenerator, blocks: &BlockStack) -> Instruction {
    let block = blocks.get(0);

    let mut b = InstructionBuilder::new();

    match &block.labels {
        BlockLabels::Loop => {
            b.push_trace(b.next());
        }
        BlockLabels::If { else_label, .. } => {
            b.pop(["cond"]);

            b.set_next(select(
                math::is_not_zero(rt, var("cond")),
                else_label.clone(),
                b.next(),
            ));
        }
        _ => {}
    }

    let param_count = block.block_type.param_count;

    b.pop((0..param_count).map(|i| format!("p{i:x}")));

    b.push_stack_frame();

    b.push((0..param_count).map(|i| var(format!("p{i:x}"))));

    b.build()
}

pub fn end_block(_rt: &mut RuntimeGenerator, blocks: &BlockStack) -> Instruction {
    let block = blocks.get(0);

    let mut b = InstructionBuilder::new();

    let result_count = block.block_type.result_count;

    b.pop((0..result_count).map(|i| format!("r{i:x}")));

    b.drop_stack_frame();

    b.push((0..result_count).map(|i| var(format!("r{i:x}"))));

    match block.labels {
        BlockLabels::Loop => {
            b.drop_trace();
        }
        BlockLabels::Func { .. } => {
            b.drop_locals_frame();
            b.ret();
        }
        _ => {}
    }

    b.build()
}

pub fn block_else(_rt: &mut RuntimeGenerator, blocks: &BlockStack) -> Instruction {
    let mut b = InstructionBuilder::new();

    match &blocks.get(0).labels {
        BlockLabels::If { end_label, .. } => {
            b.set_next(end_label.clone());
        }
        _ => unreachable!(),
    }

    b.build()
}

pub fn ret(rt: &mut RuntimeGenerator, blocks: &BlockStack) -> Instruction {
    br(rt, blocks, blocks.get_outermost_index())
}

pub fn br(_rt: &mut RuntimeGenerator, blocks: &BlockStack, depth: u32) -> Instruction {
    let mut b = InstructionBuilder::new();

    let target_block = blocks.get(depth);
    let pop_count = match &target_block.labels {
        BlockLabels::Loop => target_block.block_type.param_count,
        _ => target_block.block_type.result_count,
    };

    b.pop((0..pop_count).map(|i| format!("x{i:x}")));

    for i in 0..depth {
        b.drop_stack_frame();

        if matches!(blocks.get(i).labels, BlockLabels::Loop) {
            b.drop_trace();
        }
    }

    b.push((0..pop_count).map(|i| var(format!("x{i:x}"))));

    match &target_block.labels {
        BlockLabels::Loop => {
            b.get_trace_top("loop");
            b.set_next(var("loop"));
        }
        BlockLabels::If { end_label, .. }
        | BlockLabels::Block { end_label }
        | BlockLabels::Func { end_label } => {
            b.set_next(end_label.clone());
        }
    }

    b.build()
}

pub fn br_if(rt: &mut RuntimeGenerator, blocks: &BlockStack, depth: u32) -> Instruction {
    let mut b = InstructionBuilder::new();

    b.pop(["cond"]);

    b.set_next(select(
        math::is_zero(rt, var("cond")),
        code::single(br(rt, blocks, depth)),
        b.next(),
    ));

    b.build()
}

pub fn br_table(rt: &mut RuntimeGenerator, blocks: &BlockStack, targets: &BrTable) -> Instruction {
    let mut b = InstructionBuilder::new();

    b.pop(["idx"]);

    let mut next = br(rt, blocks, targets.default());

    // Here `targets` contain parsing `Result`s.
    // The validator should catch all parsing errors, so we can just unwrap it.
    let break_targets = targets.targets().collect::<Result<Vec<_>, _>>().unwrap();

    for (i, target) in break_targets.into_iter().enumerate().rev() {
        let i = rt.num.i32_const(i as u32);
        let target = code::single(br(rt, blocks, target));
        next = select(math::equal(rt, var("idx"), i), next, target);
    }

    b.set_next(next);

    b.build()
}
