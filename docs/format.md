# WALC lambda calculus format

This is the text format used by WALC and the interpreters.

## Syntax

The only special part is that this format uses `[]` for abstractions instead of
`λ` or `\`.
That is simpler to parse and easier to debug.

Also, `;` comments are compatible with virtually any Lisp syntax highlighting
in code editors.

```ebnf
whitespace = ' ' | TAB | VT | FF | CR | LF ;
comment = ';' (not LF)* LF ;
variable = ('a'-'z' | 'A'-'Z' | '0'-'9' | '_')+ ;
abstraction = '[' variable term ']' ;
application = '(' term term ')' ;
term = variable | abstraction | application ;
```

### Examples

```
; Comment
abc _hello_ 123 ; Variables
[x y]           ; Abstraction: λx. y
(f x)           ; Application: f x

; Y combinator:
[f ([g (g g)] [x (f (x x))])]

; Construct a pair and get the 0th element:
([p ((p [foo foo]) (bar bar))] [item0[item1 item0]])
```

See more examples in the [examples directory](../examples/lambda-calculus/).

## Interpretation

The input lambda expression must not contain free (unbound) variables,
must be evaluated [lazily](https://en.wikipedia.org/wiki/Lazy_evaluation)
(i.e. using [call by name](https://en.wikipedia.org/wiki/Evaluation_strategy#Call_by_name))
and must evaluate to a [`command`](#command).

On execution step the interpreter performs an I/O operation according to the
command and continues to the next command derived from the current one.

### Definitions

The angle-bracket notation (e.g. `f<a,b,c>`) denotes a generic definition
or a substitution into a definition.

#### Command

`command` is `optional<either<output_command, input_command>>`

If the command is `none`, the interpreter stops.
Otherwise, it executes the given command (either output or input).

#### Output command

`output_command<byte, command>` is `pair<byte, command>`

When executing an output command, the interpreter writes the byte (the 0th
item of the pair) to STDOUT and proceeds interpreting the 1st item of
the pair.

#### Input command

`input_command` is `[optional_input_byte command]`

When executing an input command, the interpreter reads one byte from STDIN,
constructs an `optional` out of it (`none` is used to indicate EOF),
applies it to the command and proceeds interpreting the result.

#### Byte

`byte<bit7,bit6,bit5,bit4,bit3,bit2,bit1,bit0>` is
`[g ((((((((g bit7) bit6) bit5) bit4) bit3) bit2) bit1) bit0)]`

This bit order was chosen to match the natural (big-endian) way of writing
numbers, so that `00110011` (51) is `[g ((((((((g 0)0)1)1)0)0)1)1)]`.

#### Either

`either<a,b>` is either:
* `left<a>`: `pair<0, a>`
* `right<b>`: `pair<1, b>`

#### Optional

`optional<a>` is either:
* `none`: `pair<0, unreachable>`
* `some<a>`: `pair<1, a>`

#### Pair

`pair<x0,x1>` is `[f ((f x0) x1)]`.

To get the 0th or the 1st element of a pair, just apply `0` or `1` to it:
`(my_pair 0)`, `(my_pair 1)`.

#### Bit

`bit` is either:
* `0`: `[x0[x1 x0]]`
* `1`: `[x0[x1 x1]]`

This order was chosen for consistency: function arguments are always written
from left to right, so it's `[x0[x1[x2...]]]` and `(((f x0)x1)x2)...`.

#### Unreachable

`unreachable` is anything that should not be executed,
often defined as some sort of a garbage value, e.g. `[_ _]`.

This is only used to fill in places that structurally
require a value when the value is not important.

### Examples

See example interpreters written in several programming languages in
the [examples directory](../examples/interpreter/).
