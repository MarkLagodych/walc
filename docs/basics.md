# Lambda calculus basics

Here are just a few short notes about how the most basic things are implemented
in WALC. The notation here is also used in the source code comments.

See the foundational definitions in the [WALC format description](./format.md).

Syntactical constructs (e.g. `let .. in ..`) are similar to those in
functional programming languages like ML, OCaml, or Haskell.

Abstractions are denoted as `x -> y`.
Applications are denoted as `(x y)` (the parentheses are mandatory)
and can be sugarized, i.e. `((a b) c)` can written as `(a b c)`.

## Let-in

```
let x = something in y
```
corresponds to this:
```
(x->y something)
```

## If-else

Assuming `cond` is a `bit` and `a` and `b` are values that need to be selected
based on the value of `a`:

```
if cond then a else b
```
corresponds to:
```
(cond b a)
```

A `1`/true bit will select the "then" branch resulting in `a`.
A `0`/false bit will select the "else" branch resulting in `b`.

## Recursion

In λ-calculus, abstractions cannot refer to themselves.
However, that does not mean that recursion is impossible:

```
let f = x -> y -> z ->
    ...use f...
in
    ...(f 1 2 3)...
```
corresponds to:
```
let f = f -> x -> y -> z ->
    ...use (f f)...
in
    ...((f f) 1 2 3)...
```

The key is to always use the function `f` applying it to itself: `(f f)`.
This way it can always refer to itself by its first argument.

For convenience, recursive functions are declared with `let_rec`:

```
let_rec f = x -> y -> z ->
    ...use f...
in
    ...(f 1 2 3)...
```

You might have heard of the Y combinator which can serve a similar purpose:

```
let Y = f -> (x->(f (x x)) x->(f (x x))) in
let f = (Y
    f -> x -> y -> z ->
        ...use f...
)
in
    ...(f 1 2 3)...
```

However, WALC does not use it at all because of the redundant evaluation steps
that it introduces.

## Binary trees

Example:

```
let tree = (pair (pair 0 1) (pair 2 3))
```

represents a tree:

```
  /\
 /  \
/\  /\
0 1 2 3
```

The tree can be indexed with a two-bit big-endian bit sequence,
i.e. an item at index 2 (`10` in BE binary) can be accessed with:
```
((tree) 1 0)
```

