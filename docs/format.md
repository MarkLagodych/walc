# WALC lambda calculus format

This is the text format used by WALC and the interpreters.

## Syntax

```ebnf
whitespace = ' ' | TAB | VT | FF | CR | LF ;
comment = '#' (not LF)* LF ;
variable = ('a'-'z' | 'A'-'Z' | '0'-'9' | '_')+ ;
abstraction = '\' variable '.' term ;
application = '(' term term ')' ;
term = variable | function | application ;
```

Examples:

```
# Comment
abc _hello_ 123 # Variables
\x. x            # Abstraction
(y y)           # Application

# Y combinator
\f. ( \x.(f(x x)) \x.(f(x x)) )
```

Backslashes `\` are used instead of lambdas `λ` for ASCII compatibility,
they are simply easier to type on different computers.

Omitting parentheses in applications (writing `x y z` instead of `((x y) z)`)
is not allowed for ease of parsing.

Notice that putting parentheses around abstractions (`(\x. x)`) is not possible
because they are reserved for applications.

## Interpretation

The input lambda expression must evaluate to a `program`.

On every step the interpreter performs an I/O operation according to the
output string and supplies the resulting input string back to the next program
function.

### Definitions

The angle-bracket notation (e.g. `f<a,b,c>`) denotes a generic definition
or a substitution into a definition.

* `unreachable` is anything that should not be executed, e.g.
    `__UNDEFINED_VARIABLE_WAKA_WAKA_1234__`.

    This is only used to fill in places that structurally
    require a value when the value is not important.

* `bit` is either:
    * `0` (`\x0.\x1.x0`)
    * `1` (`\x0.\x1.x1`).

* `byte` is `\g. ((((((((g bit0) bit1) bit2) bit3) bit4) bit5) bit6) bit7)`

* `pair<x0,x1>` is `\f. ((f x0) x1)`.

    To get the 0th or the 1st element of a pair, just apply `0` or `1` to it:
    `(my_pair 0)`, `(my_pair 1)`.

* `optional<a>` is `pair<bit,a>` and is either:
    * `some<a>`: `pair<1,a>`
    * `none`: `pair<0,unreachable>`

* `either<a,b>` is `pair<bit, a|b>` and is one of:
    * `left<a>`: `pair<0,a>`
    * `right<b>`: `pair<1,b>`

    This is a sum-type that stores a value of one of the two possible types.

* `output_command` is `pair<byte, program>`

    When executing an output command, the interpreter writes the byte to STDOUT
    and proceeds interpreting the 1st item of the pair.

* `input_command` is `\optional_input_byte. program`

    When executing a read command, the interpreter reads one byte from STDIN,
    applies it to the `input_command` (or `none` if failed to read from STDIN)
    and proceeds interpreting the result.

* `program` is `optional<either<output_command,input_command>>`

    If the program is `none`, the interpreter finishes.
    Otherwise, it executes the given command (either input or output).
