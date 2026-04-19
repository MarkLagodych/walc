// Compile with: $ gcc lambda.c -o bin/lambda -std=c99 -O3

// This is a blazingly-fast™ lambda calculus interpreter based on the WALC
// binary format. Written in C99, runs even on a potato™.

// Copyright (c) 2026 Mark Lagodych
// SPDX-License-Identifier: MIT

#include <stdint.h>
#include <stdbool.h>

#include <stdlib.h>
#include <string.h>
#include <stdio.h>

static void error(char const *message) {
    fprintf(stderr, "Error: %s\n", message);
    exit(EXIT_FAILURE);
}

// Bit fields: [31]=is_variable, [30]=is_abstraction, [29..0]=payload
typedef uint32_t term;

static inline bool term_is_variable(term t)    { return t & 0x80000000; }
static inline bool term_is_abstraction(term t) { return t & 0x40000000; }
static inline uint32_t term_payload(term t)    { return t & 0x3FFFFFFF; }

static inline term term_variable(uint32_t var)    { return var | 0x80000000; }
static inline term term_abstraction(uint32_t var) { return var | 0x40000000; }
static inline term term_application(uint32_t arg_index) { return arg_index; }

// Index into program.terms
typedef uint32_t term_id;

typedef uint32_t variable_id;

struct env;

// This does not count towards the env's ref count.
// It is a simple pointer which can be trivially copied.
typedef struct env *env_weak_ref;
// This counts towards the env's ref count.
// This can be used whenever a weak reference is required, however
// making a strong copy requires a call to env_reference().
typedef struct env *env_strong_ref;

// This cannot be copied due to the `env` field.
// Either use closure_clone() or move the value.
struct closure {
    env_strong_ref env;

    term_id term;
};

// Abstraction environment: represents an abstraction variable being bound to
// a value and being visible to all child subterms of the abstraction.
struct env {
    env_strong_ref parent;

    struct closure value;

    variable_id variable;

    uint32_t ref_count;
};

struct program {
    uint32_t term_count;
    uint32_t variable_count;

    // Contains term_count items
    term *terms;

    // variable_id -> env that binds that variable, if there is exactly one
    // such env (and if it happens to be stored here).
    //
    // Contains variable_count items. Initialized with NULLs.
    env_weak_ref *shortcut_envs;
    // variable_id -> number of currently existing envs that bind that variable.
    // Contains variable_count items. Initialized with zeroes.
    uint32_t *env_counts;
};

static inline void env_register(struct program *prog, env_strong_ref env) {
    prog->env_counts[env->variable]++;

    if (prog->shortcut_envs[env->variable] == NULL)
        prog->shortcut_envs[env->variable] = env;
}

static inline void env_unregister(struct program *prog, env_strong_ref env) {
    prog->env_counts[env->variable]--;

    if (prog->shortcut_envs[env->variable] == env)
        prog->shortcut_envs[env->variable] = NULL;
}

static inline bool env_can_use_shortcut(struct program *prog, variable_id var) {
    return prog->env_counts[var] == 1
        && prog->shortcut_envs[var] != NULL;
}

static inline env_weak_ref env_get_shortcut(
    struct program *prog,
    variable_id var
) {
    return prog->shortcut_envs[var];
}

// `opt_env` can be NULL.
static inline env_strong_ref env_reference(env_weak_ref opt_env) {
    if (opt_env) opt_env->ref_count++;
    return opt_env;
}

static void env_free(struct program *prog, env_strong_ref env);

// `opt_env` can be NULL.
static inline void env_unreference(
    struct program *prog,
    env_strong_ref opt_env
) {
    if (!opt_env) return;

    opt_env->ref_count--;
    if (opt_env->ref_count == 0) {
        env_free(prog, opt_env);
    }
}

// `opt_parent` can be NULL.
static env_strong_ref env_new(
    struct program *prog,
    env_weak_ref opt_parent,
    variable_id variable,
    struct closure value
) {
    struct env *env = malloc(sizeof(*env));
    if (!env) error("out of memory");

    *env = (struct env) {
        .parent = env_reference(opt_parent),
        .variable = variable,
        .value = value,
        .ref_count = 1,
    };

    env_register(prog, env);

    return env;
}

static inline struct closure closure_new(env_weak_ref env, term_id term) {
    return (struct closure) {
        .env = env_reference(env),
        .term = term,
    };
}

static inline struct closure closure_clone(struct closure c) {
    return (struct closure) {
        .env = env_reference(c.env),
        .term = c.term,
    };
}

static inline void closure_free(struct program *prog, struct closure c) {
    env_unreference(prog, c.env);
}

static void env_free(struct program *prog, struct env *env) {
    env_unregister(prog, env);

    // These two calls may possibly lead to very deep recursion
    env_unreference(prog, env->parent);
    closure_free(prog, env->value);

    free(env);
}

static inline bool env_is_fully_evaluated(
    struct program *prog,
    env_weak_ref env
) {
    return term_is_abstraction(prog->terms[env->value.term]);
}

static env_weak_ref env_find(
    struct program *prog,
    variable_id var,
    env_weak_ref current_env
) {
    if (env_can_use_shortcut(prog, var))
        return env_get_shortcut(prog, var);

    env_weak_ref env = current_env;
    // There is no NULL check here because the program must be well-formed.
    for (; env->variable != var; env = env->parent) ;

    return env;
}

#define BUILTIN_TERM_COUNT     (15)
#define BUILTIN_VARIABLE_COUNT (10)

// Include this inside any function that uses built-in term constants.
#define BUILTIN_USE_CONSTANTS(PROG) \
    uint32_t t = (PROG)->term_count - BUILTIN_TERM_COUNT; \
    uint32_t v = (PROG)->variable_count - BUILTIN_VARIABLE_COUNT; \
    (void) t; (void) v;

// Built-in variable IDs. `v` is the count of program's own variables.
#define BUILTIN_VAR_X0     (v+0)
#define BUILTIN_VAR_X1     (v+1)
#define BUILTIN_VAR_Y0     (v+3)
#define BUILTIN_VAR_Y1     (v+4)
#define BUILTIN_VAR_G      (v+5)
#define BUILTIN_VAR_A      (v+6)
#define BUILTIN_VAR_B      (v+7)
#define BUILTIN_VAR_FUNC   (v+8)
#define BUILTIN_VAR_ARG    (v+9)

// Built-in term indexes. `t` is the count of the program's own terms.
#define BUILTIN_TERM_0     (t+0)
#define BUILTIN_TERM_1     (t+3)
#define BUILTIN_TERM_PAIR  (t+6)
#define BUILTIN_TERM_APPLY (t+12)

// Writes built-in expression building blocks after the end of the program's
// code so that all built-in terms can be evaluated in the same way as usual
// program code.
static void builtin_init(struct program *prog) {
    BUILTIN_USE_CONSTANTS(prog)
    term *T = prog->terms;

    // 0 = [x0 [x1 x0]]
    T[BUILTIN_TERM_0]   = term_abstraction(BUILTIN_VAR_X0);
    T[BUILTIN_TERM_0+1] = term_abstraction(BUILTIN_VAR_X1);
    T[BUILTIN_TERM_0+2] = term_variable(BUILTIN_VAR_X0);

    // 1 = [y0 [y1 y1]]
    T[BUILTIN_TERM_1]   = term_abstraction(BUILTIN_VAR_Y0);
    T[BUILTIN_TERM_1+1] = term_abstraction(BUILTIN_VAR_Y1);
    T[BUILTIN_TERM_1+2] = term_variable(BUILTIN_VAR_Y1);

    // pair = [g ((g a) b)]
    // `a` and `b` are manually define by creating the environment
    T[BUILTIN_TERM_PAIR]   = term_abstraction(BUILTIN_VAR_G);
    T[BUILTIN_TERM_PAIR+1] = term_application(BUILTIN_TERM_PAIR+5);
    T[BUILTIN_TERM_PAIR+2] = term_application(BUILTIN_TERM_PAIR+4);
    T[BUILTIN_TERM_PAIR+3] = term_variable(BUILTIN_VAR_G);
    T[BUILTIN_TERM_PAIR+4] = term_variable(BUILTIN_VAR_A);
    T[BUILTIN_TERM_PAIR+5] = term_variable(BUILTIN_VAR_B);

    // apply = (func arg)
    // `func` and `arg` are manually define by creating the environment
    T[BUILTIN_TERM_APPLY]   = term_application(BUILTIN_TERM_APPLY+2);
    T[BUILTIN_TERM_APPLY+1] = term_variable(BUILTIN_VAR_FUNC);
    T[BUILTIN_TERM_APPLY+2] = term_variable(BUILTIN_VAR_ARG);
}


static void program_read(struct program *prog, char const *path) {
    FILE *f = fopen(path, "rb");
    if (!f) error("cannot open input file");

    size_t c;
    c = fread(&prog->term_count, sizeof(prog->term_count), 1, f);
    if (c != 1) error("cannot read term count");

    c = fread(&prog->variable_count, sizeof(prog->variable_count), 1, f);
    if (c != 1) error("cannot read variable count");

    prog->terms = malloc(
        sizeof(*prog->terms) * (prog->term_count + BUILTIN_TERM_COUNT)
    );
    if (!prog->terms) error("out of memory");

    c = fread(prog->terms, sizeof(*prog->terms), prog->term_count, f);
    if (c != prog->term_count) error("cannot read terms");

    prog->term_count += BUILTIN_TERM_COUNT;
    prog->variable_count += BUILTIN_VARIABLE_COUNT;
    builtin_init(prog);

    prog->shortcut_envs = calloc(
        prog->variable_count, sizeof(*prog->shortcut_envs)
    );
    if (!prog->shortcut_envs) error("out of memory");

    prog->env_counts = calloc(
        prog->variable_count, sizeof(*prog->env_counts)
    );
    if (!prog->env_counts) error("out of memory");

    fclose(f);
}

static void program_free(struct program *prog) {
    free(prog->terms);
    free(prog->shortcut_envs);
    free(prog->env_counts);
}

struct stack {
    size_t item_size;
    size_t size;
    size_t capacity;
    uint8_t *items;
};

static inline struct stack stack_new(size_t item_size) {
    return (struct stack) {
        .item_size = item_size,
    };
}

static inline void *stack_top(struct stack *stack) {
    return stack->items + (stack->size - 1) * stack->item_size;
}

static void stack_push(struct stack *stack, void *item) {
    if (stack->size == stack->capacity) {
        stack->capacity *= 2;
        if (stack->capacity < 4) stack->capacity = 4;

        stack->items = realloc(
            stack->items, stack->capacity * stack->item_size
        );
        if (!stack->items) error("out of memory");
    }

    memcpy(
        stack->items + stack->size * stack->item_size,
        item,
        stack->item_size
    );

    stack->size++;
}

static void stack_pop(struct stack *stack) {
    stack->size--;

    if (stack->capacity >= 8 && stack->size < stack->capacity / 4) {
        stack->capacity /= 2;

        stack->items = realloc(
            stack->items, stack->capacity * stack->item_size
        );
        if (!stack->items) error("out of memory");
    }
}

static void stack_free(struct stack *stack) {
    if (stack->items)
        free(stack->items);

    stack->items = NULL;
    stack->size = 0;
    stack->capacity = 0;
}

struct unevaluated_env {
    env_strong_ref env;
    // The size of the main stack at the moment when this env was recorded.
    uint32_t stack_position;
};

static void env_ensure_will_be_evaluated(
    struct program *prog,
    struct stack *unevaluated_envs,
    struct stack *value_stack,
    env_weak_ref env
) {
    if (env_is_fully_evaluated(prog, env)) return;

    stack_push(unevaluated_envs, &(struct unevaluated_env) {
        .env = env_reference(env),
        .stack_position = value_stack->size,
    });
}

static void env_assign_evaluated_value(
    struct program *prog,
    struct stack *unevaluated_envs,
    struct stack *value_stack,
    struct closure value
) {
    while (unevaluated_envs->size > 0) {
        struct unevaluated_env *env_to_update = stack_top(unevaluated_envs);

        if (env_to_update->stack_position != value_stack->size) break;

        env_strong_ref env = env_to_update->env; // the ref is moved
        stack_pop(unevaluated_envs);

        closure_free(prog, env->value);
        env->value = closure_clone(value);

        env_unreference(prog, env);
    }
}

static struct closure eval_inner(
    struct program *prog,
    struct closure current_value,
    uint32_t depth
) {
    struct stack stack = stack_new(sizeof(struct closure));

    struct stack unevaluated_envs = stack_new(sizeof(struct unevaluated_env));

    for (;;) {
        term t = prog->terms[current_value.term];

        if (term_is_variable(t)) {

            env_strong_ref env = env_reference(
                env_find(prog, term_payload(t), current_value.env)
            );

            env_ensure_will_be_evaluated(prog, &unevaluated_envs, &stack, env);

            closure_free(prog, current_value);
            current_value = closure_clone(env->value);

            env_unreference(prog, env);

        } else if (term_is_abstraction(t)) {

            env_assign_evaluated_value(
                prog, &unevaluated_envs, &stack, current_value
            );

            if (stack.size == 0) break;

            struct closure *arg_ptr = stack_top(&stack);
            struct closure arg = *arg_ptr; // the value is moved
            stack_pop(&stack);

            env_strong_ref new_env = env_new(
                prog,
                current_value.env,
                term_payload(t),
                arg // the value is moved
            );

            closure_free(prog, current_value);
            current_value.term++;
            current_value.env = new_env; // `new_env` is moved here

        } else { // Application

            struct closure arg = closure_new(
                current_value.env,
                term_payload(t)
            );

            if (term_is_variable(prog->terms[arg.term]) && depth) {
                // arg is moved to and from the function
                arg = eval_inner(prog, arg, depth - 1);
            }

            stack_push(&stack, &arg); // arg is moved here

            current_value.term++;
        }
    }

    stack_free(&stack);
    stack_free(&unevaluated_envs);
    return current_value;
}

static inline struct closure eval(struct program *prog, struct closure value) {
    return eval_inner(prog, value, 10);
}

static inline struct closure encode_zero(struct program *prog) {
    BUILTIN_USE_CONSTANTS(prog)
    return closure_new(NULL, BUILTIN_TERM_0);
}

static inline struct closure encode_one(struct program *prog) {
    BUILTIN_USE_CONSTANTS(prog)
    return closure_new(NULL, BUILTIN_TERM_1);
}

static inline struct closure encode_bit(struct program *prog, bool b) {
    return b ? encode_one(prog) : encode_zero(prog);
}

static struct closure encode_pair(
    struct program *prog,
    struct closure a,
    struct closure b
) {
    BUILTIN_USE_CONSTANTS(prog)

    env_strong_ref env_a = env_new(
        prog,
        NULL,
        BUILTIN_VAR_A,
        a
    );

    env_strong_ref env_b = env_new(
        prog,
        env_a,
        BUILTIN_VAR_B,
        b
    );

    struct closure result = closure_new(env_b, BUILTIN_TERM_PAIR);

    env_unreference(prog, env_a);
    env_unreference(prog, env_b);

    return result;
}

static struct closure apply(
    struct program *prog,
    struct closure func,
    struct closure arg
) {
    BUILTIN_USE_CONSTANTS(prog)

    env_strong_ref env_func = env_new(
        prog,
        NULL,
        BUILTIN_VAR_FUNC,
        func
    );

    env_strong_ref env_arg = env_new(
        prog,
        env_func,
        BUILTIN_VAR_ARG,
        arg
    );

    struct closure result = closure_new(env_arg, BUILTIN_TERM_APPLY);

    env_unreference(prog, env_func);
    env_unreference(prog, env_arg);

    return result;
}

static inline struct closure encode_left(
    struct program *prog,
    struct closure value
) {
    return encode_pair(prog, encode_zero(prog), value);
}

static inline struct closure encode_right(
    struct program *prog,
    struct closure value
) {
    return encode_pair(prog, encode_one(prog), value);
}

static inline struct closure encode_none(struct program *prog) {
    return encode_left(prog, encode_zero(prog));
}

static inline struct closure encode_some(
    struct program *prog,
    struct closure value
) {
    return encode_right(prog, value);
}

static inline struct closure encode_empty(struct program *prog) {
    return encode_none(prog);
}

static inline struct closure encode_cons(
    struct program *prog,
    struct closure head,
    struct closure tail
) {
    return encode_some(prog, encode_pair(prog, head, tail));
}

static struct closure encode_byte(struct program *prog, uint8_t byte) {
    struct closure result = encode_empty(prog);

    for (int i = 0; i < 8; i++) {
        struct closure bit = encode_bit(prog, byte & 1);
        byte >>= 1;

        result = encode_cons(prog, bit, result);
    }

    return result;
}

// input is either a byte (0..255) or end of file (-1)
static struct closure encode_input(struct program *prog, int32_t input) {
    if (input < 0)
        return encode_none(prog);

    return encode_some(prog, encode_byte(prog, (uint8_t)input));
}

// Returns the next command
static struct closure perform_input(
    struct program *prog,
    struct closure input_command
) {
    int32_t input = (int32_t) getchar();
    struct closure encoded_input = encode_input(prog, input);
    return apply(prog, input_command, encoded_input);
}

static bool decode_bit(struct program *prog, struct closure value) {
    BUILTIN_USE_CONSTANTS(prog)

    value = apply(prog, value, encode_zero(prog));
    value = apply(prog, value, encode_one(prog));
    value = eval(prog, value);

    if (value.env != NULL)
        error("expected bit, got something else");

    bool bit = false;
    if (value.term == BUILTIN_TERM_1)
        bit = true;
    else if (value.term == BUILTIN_TERM_0)
        bit = false;
    else
        error("expected bit, got something else");

    closure_free(prog, value);

    return bit;
}

static void decode_pair(
    struct program *prog,
    struct closure value,
    struct closure *out_first,
    struct closure *out_second
) {
    BUILTIN_USE_CONSTANTS(prog)

    value = eval(prog, value);

    *out_first = apply(prog, closure_clone(value), encode_zero(prog));
    *out_second = apply(prog, value, encode_one(prog));
}

// Returns true if the decoded value is right, false, if left.
static bool decode_either(
    struct program *prog,
    struct closure value,
    struct closure *out_value
) {
    struct closure is_right;
    decode_pair(prog, value, &is_right, out_value);
    return decode_bit(prog, is_right);
}

// Returns true if the decoded value is some, false if none.
static inline bool decode_option(
    struct program *prog,
    struct closure value,
    struct closure *out_value
) {
    bool is_some = decode_either(prog, value, out_value);

    if (!is_some)
        closure_free(prog, *out_value);

    return is_some;
}

// Returns true if list is cons, false if empty.
static bool decode_list(
    struct program *prog,
    struct closure value,
    struct closure *out_head,
    struct closure *out_tail
) {
    struct closure pair;
    if (!decode_option(prog, value, &pair))
        return false;

    decode_pair(prog, pair, out_head, out_tail);
    return true;
}

static uint8_t decode_byte(struct program *prog, struct closure value) {
    uint8_t result = 0;

    struct closure tail = value;

    for (int i = 0; i < 8; i++) {
        struct closure bit;
        if (!decode_list(prog, tail, &bit, &tail))
            error("not enough bits in a byte (unexpected end of bit list)");

        result <<= 1;
        if (decode_bit(prog, bit))
            result |= 1;
    }

    closure_free(prog, tail);

    return result;
}

static uint8_t decode_output(
    struct program *prog,
    struct closure output_command,
    struct closure *out_next_command
) {
    struct closure output;
    decode_pair(prog, output_command, &output, out_next_command);
    return decode_byte(prog, output);
}

// Returns the next command
static struct closure perform_output(
    struct program *prog,
    struct closure output_command
) {
    struct closure next_command;
    uint8_t byte = decode_output(prog, output_command, &next_command);

    putchar((char) byte);
    fflush(stdout);

    return next_command;
}

// Returns true if needs to perform I/O, false if needs to exit.
static bool decode_command(
    struct program *prog,
    struct closure command,
    struct closure *input_or_output
) {
    return decode_option(prog, command, input_or_output);
}

// Returns the next command.
static struct closure perform_io(
    struct program *prog,
    struct closure input_or_output
) {
    if (decode_either(prog, input_or_output, &input_or_output))
        return perform_input(prog, input_or_output);
    else
        return perform_output(prog, input_or_output);
}

char const *help_message =
    "Lambda calculus interpreter based on the WALC binary format\n"
    "Usage: lambda INPUT.bin\n";

int main(int argc, char **argv) {
    if (argc != 2 || strcmp(argv[1], "--help") == 0) {
        printf("%s", help_message);
        return EXIT_SUCCESS;
    }

    struct program prog;
    program_read(&prog, argv[1]);

    struct closure command = closure_new(NULL, 0);

    for (;;) {
        struct closure input_or_output;
        // `command` is moved here
        if (!decode_command(&prog, command, &input_or_output))
            break;

        command = perform_io(&prog, input_or_output);
    }

    program_free(&prog);
    return EXIT_SUCCESS;
}


// TODO fix memory leaks
