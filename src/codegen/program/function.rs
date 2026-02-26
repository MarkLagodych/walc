use crate::{analyzer::*, codegen::*};

use program::runtime::{InstructionInfo, RuntimeGenerator};

/// No control flow instructions that operate on blocks need parameter/result types, just the counts
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
            BlockType::FuncType(type_id) => {
                let func_type = &types.get_type(*type_id);
                Self::from_func_type(func_type)
            }
        }
    }
}

pub enum BlockLabelInfo {
    Func {
        end_label: code::Code,
    },
    Loop,
    If {
        else_label: code::Code,
        end_label: code::Code,
    },
    Block {
        end_label: code::Code,
    },
}

pub struct BlockInfo {
    pub label_info: BlockLabelInfo,
    pub type_info: BlockTypeInfo,
}

impl BlockInfo {
    fn from_func(func: &Func, end_label: code::Code) -> Self {
        Self {
            label_info: BlockLabelInfo::Func { end_label },
            type_info: BlockTypeInfo::from_func_type(&func.func_type),
        }
    }

    fn from_loop(blockty: &BlockType, types: &GlobalTypeInfo) -> Self {
        Self {
            label_info: BlockLabelInfo::Loop,
            type_info: BlockTypeInfo::from_block_type(blockty, types),
        }
    }

    fn from_if(
        blockty: &BlockType,
        types: &GlobalTypeInfo,
        else_label: code::Code,
        end_label: code::Code,
    ) -> Self {
        Self {
            label_info: BlockLabelInfo::If {
                else_label,
                end_label,
            },
            type_info: BlockTypeInfo::from_block_type(blockty, types),
        }
    }

    fn from_block(blockty: &BlockType, types: &GlobalTypeInfo, end_label: code::Code) -> Self {
        Self {
            label_info: BlockLabelInfo::Block { end_label },
            type_info: BlockTypeInfo::from_block_type(blockty, types),
        }
    }
}

#[derive(Default)]
pub struct BlockStack {
    blocks: Vec<BlockInfo>,
}

impl BlockStack {
    pub fn new() -> Self {
        Self::default()
    }

    fn pop(&mut self) {
        self.blocks.pop();
    }

    fn push(&mut self, block_info: BlockInfo) {
        self.blocks.push(block_info);
    }

    pub fn get(&self, level: u32) -> &BlockInfo {
        let idx = self.blocks.len() - 1 - level as usize;
        &self.blocks[idx]
    }
}

pub fn function(rt: &mut RuntimeGenerator, func: &Func, types: &GlobalTypeInfo) -> code::Code {
    let mut code = code::CodeBuilder::new();

    let mut blocks = BlockStack::new();

    let function_end_label = code.make_label();

    blocks.push(BlockInfo::from_func(func, function_end_label.clone()));

    code.push(rt.enter(func));

    // Ignore the last "end" operator that ends the function
    for op in &func.operators[..func.operators.len() - 1] {
        match op {
            Operator::Loop { blockty } => {
                blocks.push(BlockInfo::from_loop(blockty, types));
            }
            Operator::If { blockty } => {
                let else_label = code.make_label();
                let end_label = code.make_label();
                blocks.push(BlockInfo::from_if(blockty, types, else_label, end_label));
            }
            Operator::Block { blockty } => {
                let end_label = code.make_label();
                blocks.push(BlockInfo::from_block(blockty, types, end_label));
            }
            Operator::Else => {
                if let BlockLabelInfo::If { else_label, .. } = &blocks.get(0).label_info {
                    code.push_label(else_label.clone());
                } else {
                    unreachable!();
                }
            }
            Operator::End => match &blocks.get(0).label_info {
                BlockLabelInfo::If { end_label, .. } | BlockLabelInfo::Block { end_label } => {
                    code.push_label(end_label.clone());
                }
                _ => {}
            },
            _ => {}
        }

        let instr = rt.instruction(&mut InstructionInfo {
            op,
            types,
            blocks: &blocks,
        });

        code.push(instr);

        if let Operator::End = op {
            blocks.pop();
        }
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

pub struct EntrypointInfo<'a> {
    pub main_id: FuncId,
    pub start_id: Option<FuncId>,
    pub data_memory_offsets: &'a [u32],
}

pub fn entrypoint(rt: &mut RuntimeGenerator, info: &EntrypointInfo) -> io_command::IoCommand {
    let mut code = code::CodeBuilder::new();

    for (data_id, target_offset) in info.data_memory_offsets.iter().enumerate() {
        // TODO init data segments
    }

    if let Some(start_id) = info.start_id {
        code.push(rt.call(start_id));
    }

    code.push(rt.call(info.main_id));

    code.push(rt.exit());

    code.build()
}
