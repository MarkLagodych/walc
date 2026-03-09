use super::*;

/// The result will be as wide as `a` is.
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

/// Multiplies two numbers.
/// `a` and `b` must be of the same bit width. The result is of the same width.
pub fn mul(rt: &mut RuntimeGenerator, a: number::Number, b: number::Number) -> number::Number {
    if !rt.has("MUL") {
        // Numbers are in LE binary here.
        // `*` is a multiplication (bit-and) with a single bit.
        // [x, y, z, ...] denotes a list.
        //
        // Aaaaaaaa x Bbbbbbbb
        // = A * Bbbbbbbb + 0aaaaaaa x Bbbbbbbb
        // = [A * B, A * bbbbbbb + aaaaaaa x Bbbbbbbb]
        // = if A then [B, bbbbbbb + aaaaaaa x Bbbbbbbb] else [0, aaaaaaa x Bbbbbbbb]
        //
        // So, the algorithm is:
        // tail0 = aaaaaaa x Bbbbbbbb (note that this can be empty when aaaaaaa is empty)
        // tail1 = tail0 + bbbbbbb    (order is important here, the result must be of tail0 width)
        // if A then [B, tail1] else [0, tail0]

        let a = var("a");
        let a_head = list::get_head(a.clone());
        let a_tail = list::get_tail(a.clone());

        let b = var("b");
        let b_head = list::get_head(b.clone());
        let b_tail = list::get_tail(b.clone());

        let tail0 = apply(rec(var("_MUL")), [a_tail, b]);
        let tail1 = add(rt, tail0.clone(), b_tail);

        let body = select(
            list::is_not_empty(var("a")),
            list::empty(),
            select(
                a_head,
                list::node(bit(false), tail0),
                list::node(b_head, tail1),
            ),
        );

        rt.def_rec("_MUL", abs(["a", "b"], body));

        rt.def("MUL", rec(var("_MUL")));
    }

    apply(var("MUL"), [a, b])
}
