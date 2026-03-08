use super::*;

pub fn add(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> number::Number {
    if !rt.has("ADD") {
        // Full adder for two bits and a carry-in:
        // In: A, B, Cin (carry in)
        // Out: Sum, Cout (carry out)
        // 1. X = A xor B
        // 2. Sum = X xor Cin
        // 3. Cout = (A and B) or (X and Cin)

        let mut b = LetExprBuilder::new();

        let a_head = list::get_head(var("a"));
        let a_tail = list::get_tail(var("a"));

        let b_head = list::get_head(var("b"));
        let b_tail = list::get_tail(var("b"));

        b.def("x", bit_xor(a_head.clone(), b_head.clone()));

        let sum_head = bit_xor(var("x"), var("c"));

        let carry_out = bit_or(bit_and(a_head, b_head), bit_and(var("x"), var("c")));

        let sum_tail = apply(rec(var("_ADD")), [carry_out, a_tail, b_tail]);

        let body = select(
            list::is_not_empty(var("a")),
            list::empty(),
            list::node(sum_head, sum_tail),
        );

        rt.def_rec("_ADD", abs(["c", "a", "b"], b.build_in(body)));

        rt.def("ADD", apply(rec(var("_ADD")), [bit(false)]));
    }

    apply(var("ADD"), [a, b])
}

/// Fast algorithm for adding 1 to a number.
pub fn increment(rt: &mut RuntimeGenerator, a: number::Number) -> number::Number {
    if !rt.has("INC") {
        // Adding 1 is equivalent to flipping leading ones and the first zero bit. For example:
        //
        // INC(000000) = 100000   (numbers are in LE)
        //     *         *
        //
        // INC(111000) = 000100   (numbers are in LE)
        //     ^^^*      ^^^*
        //
        // INC(111111) = 000000   (numbers are in LE)
        //     ^^^^^^    ^^^^^^

        let a_head = list::get_head(var("a"));
        let a_tail = list::get_tail(var("a"));

        let body = select(
            list::is_not_empty(var("a")),
            list::empty(),
            select(
                a_head,
                list::node(bit(true), a_tail.clone()),
                list::node(bit(false), apply(rec(var("_INC")), [a_tail])),
            ),
        );

        rt.def_rec("_INC", abs(["a"], body));

        rt.def("INC", rec(var("_INC")));
    }

    apply(var("INC"), [a])
}

pub fn negate(rt: &mut RuntimeGenerator, a: number::Number) -> number::Number {
    let a_inverted = super::logical::invert(rt, a);
    increment(rt, a_inverted)
}

pub fn sub(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> number::Number {
    let b_negated = negate(rt, b);
    add(rt, a, b_negated)
}
