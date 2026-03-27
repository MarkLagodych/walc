// Compile with: $ gcc lambda.c -o bin/lambda -std=c99 -O3

// This is an efficient lambda calculus interpreter based on the WALC binary
// format, written in C99.

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

    uint32_t var_index;
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
    // If var_usage_counts[i] == 1 && unique_envs[i] != NULL, then a variable
    // with a given ID i can be accessed by indexing this array at index i.
    struct env **unique_envs;
    // Contains var_count numbers
    uint32_t *var_usage_counts;
};

struct stack {
    size_t size;
    size_t capacity;
    struct value *values;
};


void *mem_alloc_uninit(size_t size) {
    void *ptr = malloc(size);
    if (!ptr) {
        fprintf(stderr, "Error: out of memory\n");
        exit(EXIT_FAILURE);
    }

    return ptr;
}

void *mem_alloc_zeroed(size_t size) {
    void *ptr = calloc(1, size);
    if (!ptr) {
        fprintf(stderr, "Error: out of memory\n");
        exit(EXIT_FAILURE);
    }

    return ptr;
}

void *mem_resize(void *ptr, size_t new_size) {
    if (new_size == 0) {
        if (ptr)
            free(ptr);
        return NULL;
    }

    void *new_ptr = realloc(ptr, new_size);
    if (!new_ptr) {
        fprintf(stderr, "Error: out of memory\n");
        exit(EXIT_FAILURE);
    }

    return new_ptr;
}

void read_program(char const *path, struct program *prog) {
    FILE *f = fopen(path, "rb");
    if (!f) {
        fprintf(stderr, "Error: cannot open file '%s'\n", path);
        exit(EXIT_FAILURE);
    }

    fread(&prog->term_count, sizeof(prog->term_count), 1, f);
    fread(&prog->var_count, sizeof(prog->var_count), 1, f);

    prog->terms = mem_alloc_uninit(prog->term_count * sizeof(*prog->terms));

    fread(prog->terms, sizeof(*prog->terms), prog->term_count, f);

    prog->unique_envs = mem_alloc_zeroed(
        prog->var_count * sizeof(*prog->unique_envs)
    );

    prog->var_usage_counts = mem_alloc_zeroed(
        prog->var_count * sizeof(*prog->var_usage_counts)
    );

    fclose(f);
}

void free_program(struct program *prog) {
    free(prog->terms);
    free(prog->unique_envs);
    free(prog->var_usage_counts);
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
