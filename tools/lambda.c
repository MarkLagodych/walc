// Compile with: $ gcc lambda.c -o bin/lambda -std=c99 -O3

// This is a blazingly-fast™ lambda calculus interpreter based on the WALC
// binary format, written in C99.

// Copyright (c) 2026 Mark Lagodych
// SPDX-License-Identifier: MIT

#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>


typedef uint32_t term;

bool term_is_variable(term t) {
    return t & 0x80000000;
}

bool term_is_abstraction(term t) {
    return t & 0x40000000;
}

bool term_payload(term t) {
    return t & 0x3FFFFFFF;
}

struct env {
    struct env *parent;

    struct env *value_env;
    uint32_t value_term_index;

    uint32_t var_id;

    uint32_t ref_count;
};

struct value {
    struct env *env;

    // Index into program.terms
    uint32_t term_index;
};

struct program {
    uint32_t term_count;
    uint32_t var_count;

    // Contains term_count terms
    term *terms;

    // Contains var_count pointers.
    // If var_usage_counts[i] == 1 && shortcut_envs[i] != NULL, then a variable
    // with a given ID i can be accessed by indexing this array at index i.
    struct env **shortcut_envs;
    // Contains var_count numbers
    uint32_t *var_usage_counts;
};

struct value_stack {
    size_t size;
    size_t capacity;
    struct value *values;
};


void error(char const *message) {
    fprintf(stderr, "Error: %s\n", message);
    exit(EXIT_FAILURE);
}

void read_program(char const *path, struct program *prog) {
    FILE *f = fopen(path, "rb");
    if (!f) error("cannot open input file");

    fread(&prog->term_count, sizeof(prog->term_count), 1, f);
    fread(&prog->var_count, sizeof(prog->var_count), 1, f);

    prog->terms = malloc(prog->term_count * sizeof(*prog->terms));
    if (!prog->terms) error("out of memory");

    fread(prog->terms, sizeof(*prog->terms), prog->term_count, f);

    prog->shortcut_envs = calloc(
        prog->var_count, sizeof(*prog->shortcut_envs)
    );
    if (!prog->shortcut_envs) error("out of memory");

    prog->var_usage_counts = calloc(
        prog->var_count, sizeof(*prog->var_usage_counts)
    );
    if (!prog->var_usage_counts) error("out of memory");

    fclose(f);
}

void free_program(struct program *prog) {
    free(prog->terms);
    free(prog->shortcut_envs);
    free(prog->var_usage_counts);
}

void push_value(struct value_stack *stack, struct value value) {
    if (stack->size == stack->capacity) {
        stack->capacity *= 2;
        if (stack->capacity < 32) stack->capacity = 32;

        stack->values = realloc(
            stack->values, stack->capacity * sizeof(*stack->values)
        );
        if (!stack->values) error("out of memory");
    }

    stack->values[stack->size++] = value;
}

struct value pop_value(struct value_stack *stack) {
    struct value result = stack->values[--stack->size];

    if (stack->capacity >= 64 && stack->size < stack->capacity / 4) {
        stack->capacity /= 2;

        stack->values = realloc(
            stack->values, stack->capacity * sizeof(*stack->values)
        );
        if (!stack->values) error("out of memory");
    }

    return result;
}

void free_stack(struct value_stack *stack) {
    if (stack->values)
        free(stack->values);
}

bool env_is_computed(struct env *env, struct program *prog) {
    return term_is_abstraction(prog->terms[env->value_term_index]);
}

struct value value_from_env(struct env *env) {
    return (struct value) {
        .env = env,
        .term_index = env->value_term_index,
    };
}

bool var_can_use_shortcut(uint32_t var_id, struct program *prog) {
    return prog->var_usage_counts[var_id] == 1
        && prog->shortcut_envs[var_id] != NULL;
}

struct value find_var(uint32_t var_id, struct env *env, struct program *prog) {
    if (var_can_use_shortcut(var_id, prog))
        return value_from_env(prog->shortcut_envs[var_id]);

    for (; env->var_id != var_id; env = env->parent) {}

    return value_from_env(env);
}

// TODO env creation (may set shortcut env), non-recursive unreferencing
// TODO uncomputed env stack, stack of stack positions, others?

struct value eval(struct program *prog, struct value value, uint32_t depth) {
    struct value_stack stack = { 0 };

    for (;;) {
        term t = prog->terms[value.term_index];

        if (term_is_variable(t)) {
            value = find_var(term_payload(t), value.env, prog);
        } else if (term_is_abstraction(t)) {
            // TODO

            if (stack.size == 0) break;

            struct value arg = pop_value(&stack);

            // TODO allocate this
            struct env new_env = {
                .parent = value.env,
                .value_env = arg.env,
                .value_term_index = arg.term_index,
                .var_id = term_payload(t),
                .ref_count = 1,
            };

            // TODO
            value.env = &new_env;

            value.term_index++;
        } else {
            // Application
            push_value(&stack, (struct value) {
                .env = value.env,
                .term_index = term_payload(t), // Right term
            });

            value.term_index++;
        }
        // TODO use built-ins for bytes/cons/pairs/bits provided by the interp.
    }

    free_stack(&stack);
    return value;
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
    read_program(argv[1], &prog);

    // TODO

    free_program(&prog);
    return EXIT_SUCCESS;
}
