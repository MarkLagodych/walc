use super::*;

/// Applies a bitwise operation.
fn apply_bitop(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    b: number::Number,
    op: &str,
) -> number::Number {
    if !rt.has(op) {
        let a_head = list::get_head(var("a"));
        let a_tail = list::get_tail(var("a"));

        let b_head = list::get_head(var("b"));
        let b_tail = list::get_tail(var("b"));

        let head = match op {
            "AND" => bit_and(a_head, b_head),
            "OR" => bit_or(a_head, b_head),
            "XOR" => bit_xor(a_head, b_head),
            _ => unreachable!(),
        };

        rt.def_rec(
            format!("_{op}"),
            abs(["a", "b"], {
                select(
                    list::is_not_empty(var("a")),
                    list::empty(),
                    list::node(head, apply(rec(var(format!("_{op}"))), [a_tail, b_tail])),
                )
            }),
        );

        rt.def(op, rec(var(format!("_{op}"))));
    }

    apply(var(op), [a, b])
}

pub fn and(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> number::Number {
    apply_bitop(rt, a, b, "AND")
}

pub fn or(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> number::Number {
    apply_bitop(rt, a, b, "OR")
}

pub fn xor(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> number::Number {
    apply_bitop(rt, a, b, "XOR")
}
