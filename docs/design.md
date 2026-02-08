# WALC design

This document describes how WebAssembly is translated into lambda calculus.

### Code

TODO Update this to match the latest version

To be able to efficiently jump, the compiler stores all backward-addressable
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
