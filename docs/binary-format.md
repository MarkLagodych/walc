# WALC lambda calculus binary format

The binary format is used by the [C interpreter](../tools/lambda.c) to avoid
all the parsing complexity. The format is machine-dependent so that the files
can be loaded directly to the memory.

To convert, use the [text2bin tool](./tools/text2bin.ts):
```sh
tools/text2bin.ts INPUT.walc -o OUTPUT.bin
```

The file is an array of native-endian 32-bit integers with the grammar:
```ebnf
file = term_count abstraction_count term ;
term = variable | (abstraction_marker term) | (application_marker term term) ;
(* Numbers are written in big-endian here, but stored in native-endian *)
variable           = 10xxxxxx_xxxxxxxx_xxxxxxxx_xxxxxxxx ;
abstraction_marker = 01xxxxxx_xxxxxxxx_xxxxxxxx_xxxxxxxx ;
application_marker = 00xxxxxx_xxxxxxxx_xxxxxxxx_xxxxxxxx ;
```

`term_count` equals the total count of integers in the file minus 2,
i.e. excluding `term_count` and `abstraction_count` themselves.

`abstraction_count` equals the number of occurrences of the abstraction marker
in the file. This is also the total number of distinct variables in the file.

The integer payload (`xxx...`) is:
- for variables and abstractions: the unique ID of the variable.
- for applications: the position of the right term.
    Position 0 corresponds to the integer index 2 in the file.

This format is designed to be processed linearly:
to evaluate an abstraction body or an application left term,
the interpreter simply increments the current position.
