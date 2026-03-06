use crate::codegen::{
    core::*,
    runtime::{self, RuntimeGenerator},
};

use crate::analyzer::{BlockType, Func, FuncId, FuncType, GlobalTypeInfo, Operator};

/// No control flow instructions need parameter/result types, just the counts
pub struct BlockTypeInfo {
    pub param_count: u32,
    pub result_count: u32,
}

impl BlockTypeInfo {
    pub fn from_func_type(func_type: &FuncType) -> Self {
        Self {
            param_count: func_type.params().len() as u32,
            result_count: func_type.results().len() as u32,
        }
    }

    pub fn from_block_type(block_type: &BlockType, types: &GlobalTypeInfo) -> Self {
        match block_type {
            BlockType::Empty => Self {
                param_count: 0,
                result_count: 0,
            },
            BlockType::Type(_) => Self {
                param_count: 0,
                result_count: 1,
            },
            BlockType::FuncType(type_id) => Self::from_func_type(types.get_type(*type_id)),
        }
    }
}

pub enum BlockLabels {
    Func {
        end_label: code::Code,
    },
    Block {
        end_label: code::Code,
    },
    If {
        end_label: code::Code,
        else_label: code::Code,
    },
    /// `loop` performs a backward jump, i.e. it only pushes the instruction following it
    /// to the trace, so no labels are needed.
    Loop,
}

pub struct Block {
    pub labels: BlockLabels,
    pub block_type: BlockTypeInfo,
}

#[derive(Default)]
pub struct BlockStack {
    blocks: Vec<Block>,
}

impl BlockStack {
    fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, block_info: Block) {
        self.blocks.push(block_info);
    }

    fn pop(&mut self) {
        self.blocks.pop();
    }

    /// Gets the block at the given relative index counting from the innermost block,
    /// i.e. `get(0)` returns the innermost block, `get(1)` returns the next outer block, etc.
    pub fn get(&self, relative_index: u32) -> &Block {
        let idx = self.blocks.len() - 1 - relative_index as usize;
        &self.blocks[idx]
    }

    /// Gets relative index of the outermost block.
    pub fn get_outermost_index(&self) -> u32 {
        self.blocks.len() as u32 - 1
    }
}

struct FunctionBuilder<'a> {
    func: &'a Func<'a>,
    rt: &'a mut RuntimeGenerator,
    types: &'a GlobalTypeInfo,
    code: code::CodeBuilder,

    blocks: BlockStack,

    /// Stack of whether the corresponding if block has an else branch or not.
    /// If an `if` lacks an `else`, we put the `else` label right before its `end`.
    if_has_else: Vec<bool>,
}

impl<'a> FunctionBuilder<'a> {
    fn new(func: &'a Func, rt: &'a mut RuntimeGenerator, types: &'a GlobalTypeInfo) -> Self {
        Self {
            rt,
            func,
            types,
            code: code::CodeBuilder::new(),
            blocks: BlockStack::new(),
            if_has_else: Vec::new(),
        }
    }

    fn build(mut self) -> code::Code {
        self.generate_prologue_code();
        self.generate_body_code();

        self.code.build()
    }

    fn generate_prologue_code(&mut self) {
        let end_label = self.code.make_label();
        self.blocks.push(Block {
            block_type: BlockTypeInfo::from_func_type(self.func.func_type),
            labels: BlockLabels::Func { end_label },
        });

        let instr = runtime::instructions::func_prologue(self.rt, self.func);
        self.code.push(instr);
    }

    fn generate_body_code(&mut self) {
        for op in self.func.operators {
            self.before_instruction(op);

            let instr = runtime::instructions::instruction(self.rt, op, &self.blocks);
            self.code.push(instr);

            self.after_instruction(op);
        }
    }

    fn before_instruction(&mut self, op: &Operator) {
        match op {
            Operator::Loop { blockty } => {
                self.blocks.push(Block {
                    block_type: BlockTypeInfo::from_block_type(blockty, self.types),
                    labels: BlockLabels::Loop,
                });
            }

            Operator::If { blockty } => {
                let else_label = self.code.make_label();
                let end_label = self.code.make_label();
                self.blocks.push(Block {
                    block_type: BlockTypeInfo::from_block_type(blockty, self.types),
                    labels: BlockLabels::If {
                        else_label,
                        end_label,
                    },
                });

                self.if_has_else.push(false);
            }

            Operator::Block { blockty } => {
                let end_label = self.code.make_label();
                self.blocks.push(Block {
                    block_type: BlockTypeInfo::from_block_type(blockty, self.types),
                    labels: BlockLabels::Block { end_label },
                });
            }

            Operator::End => {
                let block_labels = &self.blocks.get(0).labels;

                if let BlockLabels::If { else_label, .. } = block_labels {
                    let has_else = self.if_has_else.pop().unwrap();

                    if !has_else {
                        self.code.push_label(else_label.clone());
                    }
                }

                match block_labels {
                    BlockLabels::If { end_label, .. }
                    | BlockLabels::Block { end_label }
                    | BlockLabels::Func { end_label } => {
                        self.code.push_label(end_label.clone());
                    }
                    _ => {}
                }
            }

            _ => {}
        }
    }

    fn after_instruction(&mut self, op: &Operator) {
        match op {
            Operator::End => {
                self.blocks.pop();
            }

            Operator::Else => {
                let block_labels = &self.blocks.get(0).labels;

                if let BlockLabels::If { else_label, .. } = block_labels {
                    self.code.push_label(else_label.clone());

                    let has_else = self.if_has_else.last_mut().unwrap();
                    *has_else = true;
                }
            }

            _ => {}
        }
    }
}

pub fn function(rt: &mut RuntimeGenerator, func: &Func, types: &GlobalTypeInfo) -> code::Code {
    let b = FunctionBuilder::new(func, rt, types);
    b.build()
}

pub fn input_function(rt: &mut RuntimeGenerator) -> code::Code {
    code::single(runtime::instructions::io::input_and_return(rt))
}

pub fn output_function(rt: &mut RuntimeGenerator) -> code::Code {
    code::single(runtime::instructions::io::output_and_return(rt))
}

pub fn exit_function(rt: &mut RuntimeGenerator) -> code::Code {
    code::single(runtime::instructions::io::exit(rt))
}

pub fn entrypoint(
    rt: &mut RuntimeGenerator,
    main_id: FuncId,
    start_id: Option<FuncId>,
    data_memory_offsets: impl Iterator<Item = number::I32>,
) -> io_command::IoCommand {
    let mut code = code::CodeBuilder::new();

    for (data_id, target_offset) in data_memory_offsets.enumerate() {
        code.push(runtime::instructions::memory::init_with_data(
            rt,
            var(format!("Data{data_id:x}")),
            target_offset,
        ));
    }

    if let Some(start_id) = start_id {
        code.push(runtime::instructions::control_flow::call(rt, start_id));
    }

    code.push(runtime::instructions::control_flow::call(rt, main_id));

    code.push(runtime::instructions::io::exit(rt));

    code.build()
}
