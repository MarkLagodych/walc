# WALC lambda calculus format

This is the text format used by WALC and the interpreters.

## Syntax

Text is encoded in UTF-8.

```ebnf
whitespace = ' ' | '\t' | '\v' | '\f' | '\r' | '\n' | '.' ;
comment = '#' (not '\n')* '\n' ;
variable = ('a'-'z' | 'A'-'Z' | '0'-'9' | '_')+ ;
abstraction = ('λ' | '^') variable term ;
application = '(' term term ')' ;
term = variable | function | application ;
```

The support for `.` and all versions of lambda (`λ`, `^`)
is *mandatory*.

Notice that every pair of paretheses corresponds to exactly one application.
Redundant paretheses around abstractions as well as syntax sugar like
`(a b c)` instead of `((a b) c)` are *not allowed*.

Examples:

```
# This is a comment
hello_world __WAKA_WAKA_123__ 42 1st_param # These are variables
λx y    # This is a λ-function
(f x)   # This is a function application

^x y    # Carets are allowed for ASCII compatibility
λx. y   # Dots are allowed for compatibility with existing
        # mathematical notations but are completely ignored
```

## Interpretation

The input lambda expression must evaluate to a `program`.

### Definitions

* `unreachable` is `λx x`.

    This is also sometimes called the identity function, i.e. the one
    that returns its argument unchanged.

    Here the identity function is used only to fill in places that structurally
    require a value when the value is not important.

* `bit` is either `0` (`λaλb a`) or `1` (`λaλb b`).

    `0` and `1` can be defined in a different order in other texts on LC, but
    that is not important.

* `pair<a,b>` is `λf ((f a) b)`.

    To get the 0th or the 1st element of a pair, just apply `0` or `1` to it:
    `(my_pair 0)`, `(my_pair 1)`.

* `option<a>` is `pair<bit,a>` and is either `none` (`pair<0,unreachable>`) or
    `some<a>` (`pair<1,a>`).

* `list<a,...>` is a linked list: `option<pair<a, list<...>>>`.
    An empty list (`list<>`) is `none`.

    Example:
    `list<a,b,c>` is `some<pair<a, some<pair<b, some<pair<c, none>>>>>>`.

* `byte<bit0,bit1,...bit7>` is `list<bit0,bit1,...bit7>` which is
    an unsigned 8-bit integer where `bit0` is the least significant bit.

* `string<byte0,byte1,...byteN>` is `list<byte0,byte1,...byteN>`
    which is an array of arbitrary binary data where
    `byte0` is the starting byte.

* `program` is `pair<output_string, λinput_string program | unreachable>`.

    The output string tells the interpreter what I/O operation to perform.
    If it is empty, the program ends and the 1st pair element can be
    `unreachable`.

### Commands

TODO
