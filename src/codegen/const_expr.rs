use super::*;

/// To evaluate initialization expressions for globals, pass an empty array as `globals`.
/// No globals can be referenced in those expressions (we do not support imported globals).
///
/// To evaluate initialization expressions for data segment offsets, pass the array of global
/// initializers as `globals`. The indexes of the array must match global IDs.
pub fn eval<'a>(expr: &[Operator<'a>], globals: &[Operator<'a>]) -> Operator<'a> {
    let mut stack = Vec::<Operator>::new();

    for op in expr {
        match op {
            Operator::I32Const { .. } | Operator::I64Const { .. } => stack.push(op.clone()),

            Operator::I32Add => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();

                if let (Operator::I32Const { value: a }, Operator::I32Const { value: b }) = (a, b) {
                    stack.push(Operator::I32Const { value: a + b });
                } else {
                    unreachable!()
                }
            }

            Operator::I32Sub => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();

                if let (Operator::I32Const { value: a }, Operator::I32Const { value: b }) = (a, b) {
                    stack.push(Operator::I32Const { value: a - b });
                } else {
                    unreachable!()
                }
            }

            Operator::I32Mul => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();

                if let (Operator::I32Const { value: a }, Operator::I32Const { value: b }) = (a, b) {
                    stack.push(Operator::I32Const { value: a * b });
                } else {
                    unreachable!()
                }
            }

            Operator::I64Add => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();

                if let (Operator::I64Const { value: a }, Operator::I64Const { value: b }) = (a, b) {
                    stack.push(Operator::I64Const { value: a + b });
                } else {
                    unreachable!()
                }
            }

            Operator::I64Sub => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();

                if let (Operator::I64Const { value: a }, Operator::I64Const { value: b }) = (a, b) {
                    stack.push(Operator::I64Const { value: a - b });
                } else {
                    unreachable!()
                }
            }

            Operator::I64Mul => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();

                if let (Operator::I64Const { value: a }, Operator::I64Const { value: b }) = (a, b) {
                    stack.push(Operator::I64Const { value: a * b });
                } else {
                    unreachable!()
                }
            }

            Operator::GlobalGet { global_index } => {
                stack.push(globals[*global_index as usize].clone());
            }

            Operator::End => break,

            _ => unreachable!(),
        }
    }

    stack.into_iter().next().unwrap()
}
