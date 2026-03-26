# WALC lambda calculus format

This is the text format used by WALC and the interpreters.

## Syntax

The only special part is that this format uses `[]` for abstractions instead of
`λ` or `\`. This makes it is easier to read the generated code and figure out
where each subexpression starts and ends.

Also, `;` comments are compatible with virtually any Lisp syntax highlighting
in code editors.

The whole grammar is:

```ebnf
term = variable | abstraction | application ;
variable = ('a'-'z' | 'A'-'Z' | '0'-'9' | '_')+ ;
abstraction = '[' variable term ']' ;
application = '(' term term ')' ;

whitespace = ' ' | Tab | VarticalTab | FormFeed | CarriageReturn | LineFeed ;
comment = ';' (~LineFeed)* LineFeed ;
```

### Examples

```lisp
; Comment
abc_def 12hello ; Variables
[x y]           ; Abstraction: (λx. y)
(f x)           ; Application: (f x)

; Y combinator:
[f ( [x (f (x x))] [x (f (x x))] ) ]

; Construct a pair and get the 0th element:
([p ((p foo) bar)] [item0[item1 item0]])
```

See more examples in the [examples directory](../examples/walc/).

## Semantics

The input lambda expression must not contain unbound (free) variables,
must be evaluated [lazily](https://en.wikipedia.org/wiki/Lazy_evaluation)
(i.e. using [call by need](https://en.wikipedia.org/wiki/Evaluation_strategy#Call_by_need))
and must evaluate to a [`command`](#command).

At each step the interpreter performs an I/O operation according to the
command and continues to the next command derived from the current one.

Unbound variables can be supported by implementations e.g. for debugging
purposes. In such cases, any restrictions are entirely implementation-specific.

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

`byte<bit7,bit6,...,bit0>` is `list<bit7,bit6,...,bit0>`.

#### List

`list<x0,x1,x2,...>` is `optional<pair<x0, list<x1,x2,...>>>`.
Its constructors are:
* `empty`: `none`
* `cons<head, tail>`: `some<pair<head, tail>>`

#### Optional

`optional<a>` is `either<unreachable, a>`.
Its constructors are:
* `none`: `left<unreachable>`
* `some<a>`: `right<a>`

#### Either

`either<a,b>` is either:
* `left<a>`: `pair<0, a>`
* `right<b>`: `pair<1, b>`

#### Pair

`pair<x0,x1>` is `[p ((p x0) x1)]`.

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

See [interpreters](../interp/) and [example programs](../examples/walc/).

