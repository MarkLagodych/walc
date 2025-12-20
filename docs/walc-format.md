# WALC lambda calculus format

This is the text format used by WALC and the interpreters.

## Syntax

```ebnf
whitespace = ' ' | '\t' | '\v' | '\f' | '\r' | '\n' ;
comment = '#' (not '\n')* '\n' ;
variable = ('a'-'z' | 'A'-'Z' | '0'-'9' | '_')+ ;
abstraction = '^' variable term ;
application = '(' term term ')' ;
term = variable | function | application ;
```

Note that variable names can start with a digit.

Notice that every pair of paretheses corresponds to exactly one application.
Redundant paretheses around abstractions as in `(^x x)`
as well as syntax sugar like `(a b c)` instead of `((a b) c)`
are *not allowed*.
This makes it much easier to parse than traditional mathematical notation
while maintaining the relative ease of manually writing the code.
The grammar as it is written above directly corresponds to a simple predictive
recursive-descent parser.

## Interpretation

The input lambda expression must evaluate to a `program`.

On every step the interpreter performs an I/O operation according to the
output string and supplies the resulting input string back to the next program
function.

### Definitions

The angle-bracket notation (e.g. `f<a,b,c>`) is similar to templates
in C++ or perhaps generics in C#/Java/Kotlin/Rust/Swift/TypeScript
and is needed to distinguish usual lambda applications from
substitution into the definitions.

* `unreachable` is `^x x` (or anything else, really).

    This is used only to fill in places that structurally
    require a value when the value is not important.

* `bit` is either:
    * `0` (`^x0^x1 x0`)
    * `1` (`^x0^x1 x1`).

* `pair<x0,x1>` is `^f ((f x0) x1)`.

    To get the 0th or the 1st element of a pair, just apply `0` or `1` to it:
    `(my_pair 0)`, `(my_pair 1)`.

* `option<a>` is `pair<bit,a>` and is either:
    * `some<a>`: `pair<1,a>`
    * `none`: `pair<0,unreachable>`

* `list<a,...>` is `optional<pair<a, list<...>>>` and is either:
    * `cons<a,...>`: `some<pair<a, ...>>`
    * `empty`: `none`

    Example:
    `list<a,b,c>` is `cons<a,cons<b,cons<c,empty>>>`.

* `byte<bit0,bit1,...bit7>` is `list<bit0,bit1,...bit7>`

    This is an unsigned 8-bit integer where `bit0` is the least significant bit.

* `string<byte0,byte1,...byteN>` is `list<byte0,byte1,...byteN>`

    This is an array of arbitrary binary data where
    `byte0` is the starting byte.

* `program` is either:
    * `pair<output_string, ^input_string program>`
    * `pair<output_string, unreachable>` (if the output string is empty)

    The output string tells the interpreter what I/O operation to perform.
    If it is empty, the program ends and the 1st pair element can be
    `unreachable`.

### I/O Commands

The first string byte identifies the command number.
The remaining bytes are the command argument.

#### 0

Print the argument to stdout.

#### 1

Read a character from stdin.

#### 2

Read everything from stdin.
