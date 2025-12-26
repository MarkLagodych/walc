# WALC target code design

See the [WALC format](./walc-format.md).

## Lambda calculus basics

Here you can find examples of different things in λ-calculus can be done
with a stress on how they are done in WALC output.

Syntactical constructs (e.g. `let .. in`) are similar to those in
functional programming languages like ML, OCaml, or Haskell.

### Define a variable

```
let x = some_val in use_x<x>
```
corresponds to this:
```
(\x.use_x<x>  some_val)
```

### If...else...

Assuming `cond` is a `bit` and `a` and `b` are values that need to be selected
based on the value of `a`:

```
if cond then a else b
```
corresponds to this:
```
((cond b) a)
```

A `1`/true bit will select the "then" branch resulting in `a`.
A `0`/false bit will select the "else" branch resulting in `b`.

### Multiple arguments

```
let f<x,y,z> = ...
```
corresponds to:
```
let f = \x.\y.\z. ...
```

The lambda that gets the first argument `x` returns another lambda.
That lambda gets the second argument `y` and returns the third one.
The third one returns the actual value computed from `x`, `y` and `z`.
This is called "currying".

### Tuples

Fast and small, tuples serve as the underlying representation of small
data structures with just a few items (e.g. objects with fields or numbers with
bits).

```
tuple<a, b, c>
```
corresponds to:
```
\getter. (((getter a) b) c)
```

The getter function can be either one of these three:
* `\x0.\x1.\x2.x0`
* `\x0.\x1.\x2.x1`
* `\x0.\x1.\x2.x2`

To retrieve the needed item of the tuple, just apply a getter function to it:
```
(my_tuple my_getter)
```

### Recursion

In λ-calculus, abstractions cannot refer to themselves.
However, that does not disallow recursion:

```
let f<x,y,z> =
    ...use f...
in
    ... call f<1,2,3>
```
corresponds to:
```
let f = \f.\x.\y.\z.
    ...use (f f)...
in
    ... use ((((f f) 1) 2) 3)
```

The key is to always use the function `f` applying it to itself: `(f f)`.
This way it can always refer to itself by its first argument.

### Recursive loops

The idea is to implement a loop as a recursive function.

Assuming that we have `next_state<prev_state,i>`, `should_break<state>`:
```
let loop<state, i, max> =
    if i = max then
        state   # stop
    else
        if should_break<state> then
            state   # break
        else
            loop<next_state<state,i>, i+1, max>  # continue
```

To compute the loop, simply use `loop<initial_state, 0, max>`.

### Recursive program

Assuming that we have `logic<state,input>` (which produces `output`
and `next_state`) and `initial_state`:

```
# There is no initial input, so put unreachable
main = (((program program) initial_state) unreachable)

program = \program.\state.\input.
    let output, next_state = logic<state, input>
    in pair<output, ((program program) next_state)>
```

## Internal representation

### Numbers

Short, with fixed numbers of bits, mostly need access to all the bits at once.

-> tuples of bits

### Dynamic arrays

Large, variable-sized, never shrink (WASM-specific), need good random access
time.

-> binary trees

### Stacks

-> linked lists are sufficient

### Binary trees

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

### Code

Every WASM program is a list of instructions that are executed sequentially.
Thus code instructions are stored in a linked list.

However, there exist branching instructions that can jump to (almost) arbitrary
other instructions.
To be able to efficiently jump, the compiler stores all addressable
instructions in variables and constructs an array of them that is then
addressed by branching instructions at runtime.

Time complexity:
- O(1) to step to the next instruction
- O(log(#addressable_instructions)) to branch

Illustration (instructions 0..5, I0 and I3 can be jumped to):
```
I0 -> I1 -> I2 -> I3 -> I4 -> I5
^                 ^
|                 |
|                 I3_chain
I0_chain

jump table: [I0_chain, I3_chain]
```
the construction would look like this:
```
let I3_chain = list<I3,I4,I5> in
let I0_chain = list<I0,I1,I2,..._I3> in
let jump_table = array<I0_chain,I1_chain> in
let initial_state = { I0_chain, jump_table } in
let loop<state> =
    let next_state = execute_instruction<state> in
    next_state
in loop<initial_state>
```

...where `list<a,b,c,...d>` represents `cons<a, cons<b, cons<c, d>>`.

This way instructions can directly replace the current instruction pointer
with any value, either with the next one or any from the jump table.

An advantage of this approach is that it only creates
O(#addressable_instructions) variables just to represent the program code.
