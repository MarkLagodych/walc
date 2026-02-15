mod context;
use context::*;

mod arith;
use arith::*;

use crate::{analyzer::*, codegen::*};

use std::collections::BTreeMap as Map;

pub type Instruction = Expr;

struct DefCtx<'a> {
    consts: &'a mut number::ConstantDefinitionBuilder,
}

#[derive(Default)]
pub struct InstructionDefinitionBuilder {
    map: Map<String, fn(&mut DefCtx) -> Expr>,
}

impl InstructionDefinitionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an instruction definition with the given name and definition function.
    fn add_def(&mut self, name: impl ToString, def: fn(&mut DefCtx) -> Expr) {
        self.map.insert(name.to_string(), def);
    }

    pub fn build(self, consts: &mut number::ConstantDefinitionBuilder) -> DefinitionBuilder {
        let mut b = DefinitionBuilder::new();

        // TODO use ArithDefBuilder here
        let mut ctx = DefCtx { consts };

        for (def_name, def) in self.map.into_iter() {
            b.def(def_name, def(&mut ctx));
        }

        b
    }

    pub fn instruction(
        &mut self,
        op: &Operator,
        info: &FunctionInfo,
        consts: &mut number::ConstantDefinitionBuilder,
        labels: &[Expr],
    ) -> Instruction {
        use Operator::*;

        match op {
            I32Const { .. } | I64Const { .. } | F32Const { .. } | F64Const { .. } => {
                self.push(consts.with_init_value(op))
            }

            Call { function_index } => self.call(consts.id_const(*function_index as u16)),
            CallIndirect { .. } => self.call_indirect(),

            End => {
                if labels.len() == 1 {
                    // FIXME This does not work. However, if I change this to exit(), it works
                    // for minimal.wasm/walc
                    self.leave(info.function_type)
                } else {
                    // TODO
                    todo!()
                }
            }

            // TODO
            _ => todo!(),
        }
    }

    pub fn output_and_return(&mut self) -> Instruction {
        self.add_def("Output", |_| {
            instruction(|mut ctx| {
                ctx.pop("a");
                ctx.set_next(instruction(|mut ctx| {
                    ctx.ret();
                    io_command::output(var("a"), ctx.build())
                }));
                ctx.build()
            })
        });

        var("Output")
    }

    pub fn input_and_return(&mut self) -> Instruction {
        self.add_def("Input", |def_ctx| {
            instruction(|mut ctx| {
                io_command::input(abs(["inp"], {
                    ctx.def(
                        "a",
                        select(
                            optional::is_some(var("inp")),
                            def_ctx.consts.i32_const(u32::MAX),
                            var("inp"),
                        ),
                    );

                    ctx.push(var("a"));

                    ctx.build()
                }))
            })
        });

        var("Input")
    }

    pub fn exit(&mut self) -> Instruction {
        self.add_def("Exit", |_| instruction(|_| io_command::exit()));

        var("Exit")
    }

    pub fn push(&mut self, item: Expr) -> Instruction {
        self.add_def("Push", |_| {
            abs(
                ["item"],
                instruction(|mut ctx| {
                    ctx.push(var("item"));
                    ctx.build()
                }),
            )
        });

        apply(var("Push"), [item])
    }

    pub fn call(&mut self, function_id: number::Id) -> Instruction {
        self.add_def("Call", |_| {
            abs(
                ["funcid"],
                instruction(|mut ctx| {
                    ctx.call(var("funcid"));
                    ctx.build()
                }),
            )
        });

        apply(var("Call"), [function_id])
    }

    pub fn call_indirect(&mut self) -> Instruction {
        self.add_def("CallIndirect", |_| {
            instruction(|mut ctx| {
                ctx.pop("a");
                ctx.call_indirect(var("a"));
                ctx.build()
            })
        });

        var("CallIndirect")
    }

    pub fn enter(
        &self,
        func_type: &FuncType,
        local_types: &[ValType],
        consts: &mut number::ConstantDefinitionBuilder,
    ) -> Instruction {
        instruction(|mut ctx| {
            let param_count = func_type.params().len();

            // Parameters are pushed left-to-right, so we pop them right-to-left
            for i in (0..param_count).rev() {
                ctx.pop(format!("p{i}"));
            }

            let mut locals = Vec::new();
            locals.extend((0..param_count).map(|i| var(format!("p{i}"))));
            locals.extend(local_types.iter().map(|ty| consts.default_const(*ty)));

            ctx.push_frame(locals);

            ctx.build()
        })
    }

    pub fn leave(&self, func_type: &FuncType) -> Instruction {
        instruction(|mut ctx| {
            let result_count = func_type.results().len();

            for i in (0..result_count).rev() {
                ctx.pop(format!("r{i}"));
            }

            ctx.pop_frame();

            for i in 0..result_count {
                ctx.push(var(format!("r{i}")));
            }

            ctx.ret();

            ctx.build()
        })
    }
}
