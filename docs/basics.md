# Lambda calculus basics

See the [WALC format](./format.md).

Here you can find examples of different things in λ-calculus can be done
with a stress on how they are done in WALC.

Syntactical constructs (e.g. `LET .. IN`) are similar to those in
functional programming languages like ML, OCaml, or Haskell.

## Variable definitions

```
LET x = some_val IN do_something<x>
```
corresponds to this:
```
([x do_something<x>] some_val)
```

## If-Else

Assuming `cond` is a `bit` and `a` and `b` are values that need to be selected
based on the value of `a`:

```
IF cond THEN a ELSE b
```
corresponds to this:
```
((cond b) a)
```

A `1`/true bit will select the "then" branch resulting in `a`.
A `0`/false bit will select the "else" branch resulting in `b`.

## Multiple arguments

```
LET f<x,y,z> = ...
```
corresponds to:
```
LET f = [x[y[z ...]]]
```

The lambda that gets the first argument `x` returns another lambda.
That lambda gets the second argument `y` and returns the third one.
The third one returns the actual value computed from `x`, `y` and `z`.
This is called "currying".

## Tuples

Fast and small, tuples serve as the underlying representation of small
data structures with just a few items (e.g. objects with fields or numbers with
bits).

```
tuple<a, b, c>
```
corresponds to:
```
λgetter. (((getter a) b) c)
```

The getter function can be either one of these three:
* `[x0[x1[x2 x0]]]`
* `[x0[x1[x2 x1]]]`
* `[x0[x1[x2 x2]]]`

To retrieve the needed item of the tuple, just apply a getter function to it:
```
(my_tuple my_getter)
```

## Lists

`list<a,...>` is `optional<pair<a, list<...>>>` and is either:
* `cons<a,...>`: `some<pair<a, ...>>`
* `empty`: `none`

Example:
`list<a,b,c>` is `cons<a, cons<b, cons<c, empty>>>`.

To get if the list has items, use `(my_list 0)`.
To get the item, use `((my_list 1) 0)`.
To get the tail, use `((my_list 1) 1)`.

## Recursion

In λ-calculus, abstractions cannot refer to themselves.
However, that does not disallow recursion:

```
LET f<x,y,z> =
    ...use f...
IN
    ...f<1,2,3>
```
corresponds to:
```
LET f = [f[x[y[z
    ...(f f)...]]]]
IN
    ...((((f f) 1) 2) 3)
```

The key is to always use the function `f` applying it to itself: `(f f)`.
This way it can always refer to itself by its first argument.

## Recursive loops

The idea is to implement a loop as a recursive function.

Assuming that we have `next_state<prev_state,i>`, `should_break<state>`:
```
LET loop<state, i, max> =
    IF i = max then
        state   ; break
    ELSE
        loop<next_state<state,i>, i+1, max>  ; continue
```

To compute the loop, simply use `loop<initial_state, 0, max>`.

## Numbers

TODO

Short, with fixed numbers of bits, mostly need access to all the bits at once.

-> tuples of bits

## Dynamic arrays

Large, variable-sized, never shrink (WASM-specific), need good random access
time.

-> binary trees

## Stacks

-> linked lists are sufficient

## Binary trees

```
let tree<left,right> = pair<left, right>
```

The leaves are optionals.

Empty subtrees are assigned to empty dummy tree constants:

```
let dummy_tree_2 = tree<none, none>
let dummy_tree_4 = tree<dummy_tree_2, dummy_tree_2>
let dummy_tree_8 = tree<dummy_tree_4, dummy_tree_4>
...
```

To index a tree, just apply all index bits to it.
To remove from a tree, just assign its element to `none`.

TODO Update basics to match all the codegen primitives
