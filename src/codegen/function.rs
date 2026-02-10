use super::*;

use crate::analyzer::*;

pub type FunctionBody = Expr;

pub struct EnvironmentInfo<'a> {
    pub consts: &'a mut number::ConstantDefinitionBuilder,
    pub instrs: &'a mut instruction::InstructionDefinitionBuilder,
    pub types: &'a [FuncType],
}

pub struct FunctionInfo<'a> {
    pub function_type: &'a FuncType,
    pub local_types: &'a [ValType],
    pub instructions: &'a [Operator<'a>],
}

pub fn function(env: &EnvironmentInfo, func: &FunctionInfo) -> FunctionBody {
    let mut labels = Vec::<Operator>::new();

    unreachable()
}

pub fn input_function() -> FunctionBody {
    unreachable()
}

pub fn output_function() -> FunctionBody {
    unreachable()
}

pub fn exit_function() -> FunctionBody {
    io_command::exit()
}
