use super::*;

use util::UtilGenerator;

use crate::analyzer::*;

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
            type_info: BlockTypeInfo::from_func_type(func.func_type),
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

    fn push(&mut self, block_info: BlockInfo) {
        self.blocks.push(block_info);
    }

    fn pop(&mut self) {
        self.blocks.pop();
    }

    pub fn get(&self, level: u32) -> &BlockInfo {
        let idx = self.blocks.len() - 1 - level as usize;
        &self.blocks[idx]
    }

    pub fn get_depth(&self) -> u32 {
        self.blocks.len() as u32
    }
}

struct FunctionBuilder<'a> {
    func: &'a Func<'a>,
    util: &'a mut UtilGenerator,
    types: &'a GlobalTypeInfo,
    code: code::CodeBuilder,

    blocks: BlockStack,

    /// Stack of whether the corresponding if block has an else branch or not.
    /// If an `if` lacks an `else`, we put the `else` label right before its `end`.
    if_has_else: Vec<bool>,
}

impl<'a> FunctionBuilder<'a> {
    fn new(func: &'a Func, util: &'a mut UtilGenerator, types: &'a GlobalTypeInfo) -> Self {
        Self {
            util,
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
        let function_end_label = self.code.make_label();
        let block_info = BlockInfo::from_func(self.func, function_end_label.clone());
        self.blocks.push(block_info);

        let instr = self.util.func_prologue(self.func);
        self.code.push(instr);
    }

    fn generate_body_code(&mut self) {
        for op in self.func.operators {
            self.before_instruction(op);

            let instr = self.util.instruction(op, &self.blocks);
            self.code.push(instr);

            self.after_instruction(op);
        }
    }

    fn before_instruction(&mut self, op: &Operator) {
        match op {
            Operator::Loop { blockty } => {
                let block_info = BlockInfo::from_loop(blockty, self.types);

                self.blocks.push(block_info);
            }

            Operator::If { blockty } => {
                let else_label = self.code.make_label();
                let end_label = self.code.make_label();
                let block_info = BlockInfo::from_if(blockty, self.types, else_label, end_label);

                self.blocks.push(block_info);
                self.if_has_else.push(false);
            }

            Operator::Block { blockty } => {
                let end_label = self.code.make_label();
                let block_info = BlockInfo::from_block(blockty, self.types, end_label);

                self.blocks.push(block_info);
            }

            Operator::Else => match &self.blocks.get(0).label_info {
                BlockLabelInfo::If { else_label, .. } => {
                    self.code.push_label(else_label.clone());

                    *self.if_has_else.last_mut().unwrap() = true;
                }
                _ => unreachable!(),
            },

            Operator::End => match &self.blocks.get(0).label_info {
                BlockLabelInfo::If {
                    end_label,
                    else_label,
                } => {
                    let has_else = self.if_has_else.pop().unwrap();

                    if !has_else {
                        self.code.push_label(else_label.clone());
                    }

                    self.code.push_label(end_label.clone());
                }
                BlockLabelInfo::Block { end_label } | BlockLabelInfo::Func { end_label } => {
                    self.code.push_label(end_label.clone());
                }
                _ => {}
            },

            _ => {}
        }
    }

    fn after_instruction(&mut self, op: &Operator) {
        if let Operator::End = op {
            self.blocks.pop();
        }
    }
}

pub fn function(util: &mut UtilGenerator, func: &Func, types: &GlobalTypeInfo) -> code::Code {
    let b = FunctionBuilder::new(func, util, types);
    b.build()
}

pub fn input_function(util: &mut UtilGenerator) -> code::Code {
    let mut code = code::CodeBuilder::new();
    code.push(util.input_and_return());
    code.build()
}

pub fn output_function(util: &mut UtilGenerator) -> code::Code {
    let mut code = code::CodeBuilder::new();
    code.push(util.output_and_return());
    code.build()
}

pub fn exit_function(util: &mut UtilGenerator) -> code::Code {
    let mut code = code::CodeBuilder::new();
    code.push(util.exit());
    code.build()
}

pub struct EntrypointInfo<'a> {
    pub main_id: FuncId,
    pub start_id: Option<FuncId>,
    pub data_memory_offsets: &'a [u32],
}

pub fn entrypoint(util: &mut UtilGenerator, info: &EntrypointInfo) -> io_command::IoCommand {
    let mut code = code::CodeBuilder::new();

    for (data_id, target_offset) in info.data_memory_offsets.iter().enumerate() {
        // TODO init data segments
    }

    if let Some(start_id) = info.start_id {
        code.push(util.call(start_id));
    }

    code.push(util.call(info.main_id));

    code.push(util.exit());

    code.build()
}
