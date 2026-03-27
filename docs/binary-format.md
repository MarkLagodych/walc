# WALC lambda calculus binary format

The binary format is used by the [C interpreter](../tools/lambda.c) to avoid
all the parsing complexity. The format is machine-dependent so that the files
can be loaded directly to the memory.

The file is an array of native-endian 32-bit integers with the grammar:
```ebnf
file = term_count abstraction_count term ;
term = variable | (abstraction_marker term) | (application_marker term term) ;
(* Numbers are written in big-endian here, but stored in native-endian *)
variable           = 11xxxxxx xxxxxxxx xxxxxxxx xxxxxxxx ;
abstraction_marker = 10xxxxxx xxxxxxxx xxxxxxxx xxxxxxxx ;
application_marker = 00xxxxxx xxxxxxxx xxxxxxxx xxxxxxxx ;
```

The term count equals the total count of integers in the file minus 2.

The abstraction count equals the number of occurrences of the abstraction marker
in the file. This is also called the total number of unique variables in the
expression.

The integer payload (`xxx...`) is:
- for variables and abstractions: the unique ID of the variable
- for applications: the index of the right term in the term array

