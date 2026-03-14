use super::*;

/// `b` must not be shorter than `a`.
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
        // `x` is a multiplication of two numbers.
        //
        // Aaaaaaaa x Bbbbbbbb
        // = A * Bbbbbbbb + 0aaaaaaa x Bbbbbbbb
        // = cons(A * B, A * bbbbbbb + aaaaaaa x Bbbbbbbb)
        // = if A then cons(B, bbbbbbb + aaaaaaa x Bbbbbbbb) else cons(0, aaaaaaa x Bbbbbbbb)
        //
        // So, the algorithm is:
        // tail0 = aaaaaaa x Bbbbbbbb (note that this can be empty when aaaaaaa is empty)
        // tail1 = tail0 + bbbbbbb    (order is important here, the result must be of tail0 width)
        // if A then cons(B, tail1) else cons(0, tail0)

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

/// Returns a list of leading zeroes of the given number.
///
/// Example:
/// ```text
///         these are leading zeroes
///               vvv
/// a      = 00101000 (LE)
/// result =      000
/// ```
fn get_leading_zeroes(rt: &mut RuntimeGenerator, a: number::Number) -> list::List {
    if !rt.has("LEADZ") {
        let a_head = list::get_head(var("a"));
        let a_tail = list::get_tail(var("a"));

        let prev_zeroes = var("z");
        let more_zeroes = list::node(bit(false), prev_zeroes.clone());
        let empty_zeroes = list::empty();

        let recurse = |zeroes| apply(rec(var("_LEADZ")), [zeroes, a_tail.clone()]);

        let definition = abs(
            ["z", "a"],
            select(
                list::is_not_empty(var("a")),
                prev_zeroes,
                select(a_head, recurse(more_zeroes), recurse(empty_zeroes)),
            ),
        );

        rt.def_rec("_LEADZ", definition);

        rt.def("LEADZ", apply(rec(var("_LEADZ")), [list::empty()]));
    }

    apply(var("LEADZ"), [a])
}

/// Subtracts one list of zeroes (`b`) from another (`a`).
/// If `b` is longer than `a`, the result will be empty.
///
/// Example:
/// ```text
///         this is the difference
///              vv
/// a      = 000000
/// b      = 0000
/// result = 00
/// ```
///
/// ```text
/// a      = 00
/// b      = 000
/// result = (empty)
/// ```
fn sub_zeroes(rt: &mut RuntimeGenerator, a: list::List, b: list::List) -> list::List {
    if !rt.has("SUBZ") {
        let a_not_empty = list::is_not_empty(var("a"));
        let a_tail = list::get_tail(var("a"));

        let b_not_empty = list::is_not_empty(var("b"));
        let b_tail = list::get_tail(var("b"));

        let recurse = apply(rec(var("_SUBZ")), [a_tail, b_tail]);

        let definition = abs(
            ["a", "b"],
            select(
                a_not_empty,
                list::empty(),
                select(b_not_empty, var("a"), recurse),
            ),
        );

        rt.def_rec("_SUBZ", definition);

        rt.def("SUBZ", rec(var("_SUBZ")));
    }

    apply(var("SUBZ"), [a, b])
}

/// Prepends the given list of zeroes to the given number.
///
/// Example:
/// ```text
/// a      = 11111111    (LE)
/// zeroes = 000
/// result = 00011111111 (LE)
/// ```
fn prepend_zeroes(rt: &mut RuntimeGenerator, a: number::Number, zeroes: list::List) -> list::List {
    if !rt.has("PREPZ") {
        let zeroes_not_empty = list::is_not_empty(var("z"));
        let zeroes_head = list::get_head(var("z"));
        let zeroes_tail = list::get_tail(var("z"));

        let recurse_tail = apply(rec(var("_PREPZ")), [var("a"), zeroes_tail]);

        let definition = abs(
            ["a", "z"],
            select(
                zeroes_not_empty,
                var("a"),
                list::node(zeroes_head, recurse_tail),
            ),
        );

        rt.def_rec("_PREPZ", definition);

        rt.def("PREPZ", rec(var("_PREPZ")));
    }

    apply(var("PREPZ"), [a, zeroes])
}

/// Performs unsigned integer division of two numbers of the same bit width specified in `bits`.
/// `b` must not be zero and must be shifted to the left with zeroes.
/// `result_init` is the initial value of the result: 0 of the required bit width.
/// `partial_result` is the initial value of the partial result: 1 of the required bit width,
/// shifted to the left with zeroes.
fn div_helper(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    b: number::Number,
    partial_result: number::Number,
    result_init: number::Number,
) -> number::Number {
    if !rt.has("_DIV") {
        // See the comment in `div()`, the algorithm translates to:
        //
        // let rec _DIV(a, b, partial_result, result) =
        //     let b_le_a = a >= b in
        //     let a = if b_le_a then a - b else a in
        //     let result = if b_le_a then result + partial_result else result in
        //     if list_head(partial_result) then
        //         result
        //     else
        //         _DIV(a, tail(b), tail(partial_result), result)
        //
        // Note that this algorithm checks the sign of a - b rather than checking b <= a because
        // b is not of the same bit width as a, so they cannot be compared.

        let mut defs = LetExprBuilder::new();
        defs.def(
            "b_le_a",
            // The order here is important because `b` may be wider than `a`
            super::comparisons::greater_equal_unsigned(rt, var("a"), var("b")),
        );
        defs.def(
            "a",
            select(var("b_le_a"), var("a"), sub(rt, var("a"), var("b"))),
        );
        defs.def(
            "res",
            select(var("b_le_a"), var("res"), add(rt, var("res"), var("part"))),
        );
        let body = select(
            list::get_head(var("part")),
            apply(
                rec(var("_DIV")),
                [
                    var("a"),
                    list::get_tail(var("b")),
                    list::get_tail(var("part")),
                    var("res"),
                ],
            ),
            var("res"),
        );

        let definition = abs(["a", "b", "part", "res"], defs.build_in(body));

        rt.def_rec("_DIV", definition);
    }

    apply(rec(var("_DIV")), [a, b, partial_result, result_init])
}

// Performs unsigned integer division of two numbers of the same bit width specified in `bits`.
// The result is an optional number of the same width.
// If `b` is zero, the result is `None`.
fn div(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    b: number::Number,
    bits: u8,
) -> optional::Optional {
    let name = format!("DIV{bits}");
    if !rt.has(&name) {
        // Uses the long division algorithm.
        // See https://en.wikipedia.org/wiki/Division_algorithm#Long_division
        //
        // The algorithm looks like this:
        //
        // In: a, b
        // Out: a/b
        // let max_shift = abs(clz(b) - clz(a)) // "CLZ" means "Count Leading Zeroes"
        // b <<= max_shift                      // implemented by prepending zeroes
        // let partial_result = 1 << max_shift  // implemented by prepending zeroes
        // let result = 0
        // loop {
        //     if b <= a {
        //         a -= b
        //         result += partial_result
        //     }
        //     if partial_result & 1 { return result }  // "& 1" is the same as taking list head
        //     b >>= 1                           // same as taking list tail
        //     partial_result >>= 1              // same as taking list tail
        // }

        let (result_init, partial_result) = match bits {
            32 => (rt.num.i32_const(0), rt.num.i32_const(1)),
            64 => (rt.num.i64_const(0), rt.num.i64_const(1)),
            _ => unreachable!(),
        };

        let mut defs = LetExprBuilder::new();

        let a_zeroes = get_leading_zeroes(rt, var("a"));
        let b_zeroes = get_leading_zeroes(rt, var("b"));
        let shift_zeroes = sub_zeroes(rt, b_zeroes, a_zeroes);
        defs.def("z", shift_zeroes);

        let b = prepend_zeroes(rt, var("b"), var("z"));
        let partial_result = prepend_zeroes(rt, partial_result, var("z"));

        let body = select(
            super::comparisons::is_zero(rt, var("b")),
            optional::some(div_helper(rt, var("a"), b, partial_result, result_init)),
            optional::none(),
        );

        rt.def(&name, abs(["a", "b"], defs.build_in(body)));
    }

    apply(var(name), [a, b])
}

pub fn i32_div_unsigned(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    b: number::Number,
) -> optional::Optional {
    div(rt, a, b, 32)
}

pub fn i64_div_unsigned(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    b: number::Number,
) -> optional::Optional {
    div(rt, a, b, 64)
}

fn div_signed(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    b: number::Number,
    bits: u8,
) -> optional::Optional {
    let name = format!("DIVs{}", bits);

    if !rt.has(&name) {
        let mut defs = LetExprBuilder::new();

        let smallest_signed = match bits {
            32 => rt.num.i32_const(i32::MIN as u32),
            64 => rt.num.i64_const(i64::MIN as u64),
            _ => unreachable!(),
        };

        let neg_one = match bits {
            32 => rt.num.i32_const(-1i32 as u32),
            64 => rt.num.i64_const(-1i64 as u64),
            _ => unreachable!(),
        };

        // If a is -2^(bits-1) and b is -1, a/b will be out of range of representable signed numbers
        defs.def(
            "range_error",
            bit_and(
                super::comparisons::equal(rt, var("a"), smallest_signed),
                super::comparisons::equal(rt, var("b"), neg_one),
            ),
        );

        defs.def("a_neg", list::get_head(number::reverse_bits(var("a"))));
        defs.def("b_neg", list::get_head(number::reverse_bits(var("b"))));

        defs.def("a", select(var("a_neg"), var("a"), negate(rt, var("a"))));
        defs.def("b", select(var("b_neg"), var("b"), negate(rt, var("b"))));

        defs.def("res", div(rt, var("a"), var("b"), bits));

        let body = select(
            var("range_error"),
            select(
                bit_xor(var("a_neg"), var("b_neg")),
                var("res"),
                select(
                    optional::is_some(var("res")),
                    optional::none(),
                    optional::some(negate(rt, optional::unwrap(var("res")))),
                ),
            ),
            optional::none(),
        );

        let definition = abs(["a", "b"], defs.build_in(body));

        rt.def(&name, definition);
    }

    apply(var(&name), [a, b])
}

pub fn i32_div_signed(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    b: number::Number,
) -> optional::Optional {
    div_signed(rt, a, b, 32)
}

pub fn i64_div_signed(
    rt: &mut RuntimeGenerator,
    a: number::Number,
    b: number::Number,
) -> optional::Optional {
    div_signed(rt, a, b, 64)
}
