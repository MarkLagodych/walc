/*
TODO C intepreter that reads a pre-compiled binary format:
len:i32, vars:i32, expr:i32...
expr:i32: highest 2 bits indicate type (var, app, abs),
          other bits indicate the ID/index:
          for abs and var: var ID
          for app: right expression index

TODO pre-allocated envs (array of pointers + counts for each env) to avoid
long searches for global (or any non-recursively used) variables.
*/
