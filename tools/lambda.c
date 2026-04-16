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

// Bit fields: [31]=is_abstraction, [30]=is_variable, [29..0]=payload
typedef uint32_t term;

static inline bool term_is_variable(term t)    { return t & 0x80000000; }
static inline bool term_is_abstraction(term t) { return t & 0x40000000; }
static inline uint32_t term_payload(term t)    { return t & 0x3FFFFFFF; }

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

    return env_reference(env);
}

static void program_read(struct program *prog, char const *path) {
    FILE *f = fopen(path, "rb");
    if (!f) error("cannot open input file");

    size_t c;
    c = fread(&prog->term_count, sizeof(prog->term_count), 1, f);
    if (c != 1) error("cannot read term count");

    c = fread(&prog->variable_count, sizeof(prog->variable_count), 1, f);
    if (c != 1) error("cannot read variable count");

    prog->terms = malloc(sizeof(*prog->terms) * prog->term_count);
    if (!prog->terms) error("out of memory");

    c = fread(prog->terms, sizeof(*prog->terms), prog->term_count, f);
    if (c != prog->term_count) error("cannot read terms");

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
    struct stack *uncomputed_envs,
    struct stack *value_stack,
    env_weak_ref env
) {
    if (env_is_fully_evaluated(prog, env)) return;

    stack_push(uncomputed_envs, &(struct unevaluated_env) {
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

    struct stack uncomputed_envs = stack_new(sizeof(struct unevaluated_env));

    for (;;) {
        term t = prog->terms[current_value.term];

        if (term_is_variable(t)) {

            env_weak_ref env = env_find(
                prog, term_payload(t), current_value.env
            );

            env_ensure_will_be_evaluated(prog, &uncomputed_envs, &stack, env);

            closure_free(prog, current_value);
            current_value = closure_clone(env->value);

        } else if (term_is_abstraction(t)) {

            env_assign_evaluated_value(
                prog, &uncomputed_envs, &stack, current_value
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
            current_value = closure_new(new_env, current_value.term + 1);

            env_unreference(prog, new_env);

        } else { // Application

            struct closure arg = closure_new(
                current_value.env,
                term_payload(t)
            );

            if (term_is_variable(arg.term) && depth) {
                // arg is moved into the function
                arg = eval_inner(prog, arg, depth - 1);
            }

            stack_push(&stack, &arg); // arg is moved here

            current_value.term++;
        }
    }

    stack_free(&stack);
    return current_value;
}

static inline struct closure eval(struct program *prog, struct closure value) {
    return eval_inner(prog, value, 10);
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

    // TODO

    program_free(&prog);
    return EXIT_SUCCESS;
}
