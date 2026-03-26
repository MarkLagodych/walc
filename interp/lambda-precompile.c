/*
Compile with:
$ gcc lambda-precompile.c -o bin/lambda-precompile -std=c99 -O3

TODO comments
*/


#include <stdbool.h>
#include <stdint.h>

#include <stdlib.h>
#include <stdio.h>
#include <ctype.h>


bool is_varchar(char c) {
    return isalnum(c) || c == '_';
}

bool is_bracket(char c) {
    return c == '(' || c == ')' || c == '[' || c == ']';
}

// End follows after the last character, so if ptr == end, the string is empty.
struct str { char *ptr; char *end; };

bool str_empty(struct str const *s) {
    return s->ptr >= s->end;
}

size_t str_len(struct str s) {
    return s.end - s.ptr;
}

bool str_equal(struct str a, struct str b) {
    if (str_len(a) != str_len(b)) return false;

    while (!str_empty(&a)) {
        if (*a.ptr++ != *b.ptr++) return false;
    }

    return true;
}

void *alloc(size_t size) {
    void *ptr = malloc(size);
    if (!ptr) {
        fprintf(stderr, "Error: out of memory\n");
        exit(EXIT_FAILURE);
    }

    return ptr;
}

struct str read_file(FILE *f) {
    fseek(f, 0, SEEK_END);
    size_t size = ftell(f);
    rewind(f);

    struct str buffer;
    buffer.ptr = alloc(size);
    buffer.end = buffer.ptr + size;

    size_t read_count = fread(buffer.ptr, 1, size, f);
    if (read_count != size) {
        fprintf(stderr, "Error: failed to read file\n");
        exit(EXIT_FAILURE);
    }

    return buffer;
}

struct str read_var(struct str *input) {
    struct str var;
    var.ptr = input->ptr;

    while (!str_empty(input) && is_varchar(*input->ptr)) {
        input->ptr++;
    }

    var.end = input->ptr;
    return var;
}

void skip_whitespaces_and_comments(struct str *input) {
    while (!str_empty(input)) {
        if (isspace(*input->ptr)) {
            input->ptr++;
        } else if (*input->ptr == ';') {
            // Skip comments until the end of the line.
            while (!str_empty(input) && *input->ptr != '\n') {
                input->ptr++;
            }
        } else {
            break;
        }
    }
}

struct str read_token(struct str *input) {
    skip_whitespaces_and_comments(input);

    if (str_empty(input)) {
        return *input;
    }

    if (is_bracket(*input->ptr)) {
        struct str token = { input->ptr, input->ptr + 1 };
        input->ptr++;
        return token;
    } else if (is_varchar(*input->ptr)) {
        return read_var(input);
    } else {
        fprintf(stderr, "Error: unexpected character '%c'\n", *input->ptr);
        exit(EXIT_FAILURE);
    }
}

struct expr {
    union {
        struct { struct str name; } variable;
        struct { struct str param_name; struct expr *body; } abstraction;
        struct { struct expr *left; struct expr *right; } application;
    } val;

    enum { EXPR_VARIABLE, EXPR_ABSTRACTION, EXPR_APPLICATION } type;
};

void free_expr(struct expr *expr) {
    for (;expr;) {
        // Avoids recursion as much as possible
        struct expr *next = NULL;

        switch (expr->type) {
            case EXPR_VARIABLE:
                // Nothing to free.
            break;

            case EXPR_ABSTRACTION:
                next = expr->val.abstraction.body;
            break;

            case EXPR_APPLICATION:
                next = expr->val.application.left;
                free_expr(expr->val.application.right);
            break;
        }

        free(expr);
        expr = next;
    }
}

struct parser_stack_item {
    struct parser_stack_item *prev;

    union {
        struct expr *expr;
        struct str abstraction_variable;
    } val;

    enum { ITEM_EXPR, ITEM_ABSTRACTION, ITEM_APPLICATION } type;
};

void push_expr(struct parser_stack_item **current, struct expr *expr) {
    struct parser_stack_item *item = alloc(sizeof(*item));
    item->prev = *current;
    item->type = ITEM_EXPR;
    item->val.expr = expr;
    *current = item;
}

void push_abstraction(struct parser_stack_item **current, struct str variable) {
    struct parser_stack_item *item = alloc(sizeof(*item));
    item->prev = *current;
    item->type = ITEM_ABSTRACTION;
    item->val.abstraction_variable = variable;
    *current = item;
}

void push_application(struct parser_stack_item **current) {
    struct parser_stack_item *item = alloc(sizeof(*item));
    item->prev = *current;
    item->type = ITEM_APPLICATION;
    *current = item;
}

// The returned item must be freed by the caller.
struct parser_stack_item *pop_item(struct parser_stack_item **current) {
    if (!*current) {
        fprintf(stderr, "Parsing error: too few items in expression\n");
        exit(EXIT_FAILURE);
    }

    struct parser_stack_item *item = *current;
    *current = item->prev;
    return item;
}

void parse_application_start(struct parser_stack_item **current) {
    push_application(current);
}

void parse_application_end(struct parser_stack_item **current) {
    struct parser_stack_item *right = pop_item(current);
    struct parser_stack_item *left = pop_item(current);
    struct parser_stack_item *start = pop_item(current);

    if (right->type != ITEM_EXPR || left->type != ITEM_EXPR
        || start->type != ITEM_APPLICATION
    ) {
        fprintf(stderr, "Error: unexpected token ')'\n");
        exit(EXIT_FAILURE);
    }

    struct expr *left_expr = left->val.expr;
    struct expr *right_expr = right->val.expr;

    free(right);
    free(left);
    free(start);

    struct expr *app = alloc(sizeof(*app));
    app->type = EXPR_APPLICATION;
    app->val.application.left = left_expr;
    app->val.application.right = right_expr;

    push_expr(current, app);
}

void parse_abstraction_start(
    struct parser_stack_item **current, struct str *input
) {
    struct str variable = read_var(input);
    if (str_empty(&variable)) {
        fprintf(stderr, "Error: expected variable after '['\n");
        exit(EXIT_FAILURE);
    }

    push_abstraction(current, variable);
}

void parse_abstraction_end(struct parser_stack_item **current) {
    struct parser_stack_item *body = pop_item(current);
    struct parser_stack_item *start = pop_item(current);

    if (body->type != ITEM_EXPR || start->type != ITEM_ABSTRACTION) {
        fprintf(stderr, "Error: unexpected token ']'\n");
        exit(EXIT_FAILURE);
    }

    struct expr *body_expr = body->val.expr;
    struct str param_name = start->val.abstraction_variable;

    free(body);
    free(start);

    struct expr *abs = alloc(sizeof(*abs));
    abs->type = EXPR_ABSTRACTION;
    abs->val.abstraction.param_name = param_name;
    abs->val.abstraction.body = body_expr;

    push_expr(current, abs);
}

void parse_variable(struct parser_stack_item **current, struct str token) {
    struct expr *var = alloc(sizeof(*var));
    var->type = EXPR_VARIABLE;
    var->val.variable.name = token;

    push_expr(current, var);
}

struct expr *get_parser_result(struct parser_stack_item **current) {
    struct parser_stack_item *item = pop_item(current);
    if (item->type != ITEM_EXPR) {
        fprintf(stderr, "Error: unexpected end of input\n");
        exit(EXIT_FAILURE);
    }

    struct expr *result = item->val.expr;
    free(item);

    if (*current) {
        fprintf(stderr, "Error: expected more tokens\n");
        exit(EXIT_FAILURE);
    }

    return result;
}

struct expr *parse(struct str input) {
    struct parser_stack_item *stack = NULL;

    for (struct str token;
        (token = read_token(&input), !str_empty(&token));
    ) {
        switch (token.ptr[0]) {
            case '(':
                parse_application_start(&stack);
            break;

            case ')':
                parse_application_end(&stack);
            break;

            case '[':
                parse_abstraction_start(&stack, &input);
            break;

            case ']':
                parse_abstraction_end(&stack);
            break;

            default:
                parse_variable(&stack, token);
            break;
        }
    }

    return get_parser_result(&stack);
}

// TODO compile to binary format

void print(struct expr *expr) {
    switch (expr->type) {
        case EXPR_VARIABLE:
            printf("%.*s", (int)(expr->val.variable.name.end - expr->val.variable.name.ptr), expr->val.variable.name.ptr);
        break;

        case EXPR_ABSTRACTION:
            printf("[");
            printf("%.*s", (int)(expr->val.abstraction.param_name.end - expr->val.abstraction.param_name.ptr), expr->val.abstraction.param_name.ptr);
            printf(" ");
            print(expr->val.abstraction.body);
            printf("]");
        break;

        case EXPR_APPLICATION:
            printf("(");
            print(expr->val.application.left);
            printf(" ");
            print(expr->val.application.right);
            printf(")");
        break;
    }
}

char const *help_message =
    "Usage: $ lambda-precompile INPUT.walc OUTPUT.walcbin\n"
    "\n"
    "Converts lambda expressions from WALC text format to a binary format.\n"
    "The resulting files depend on machine endianness and are not portable.\n";

int main(int argc, char **argv) {
    if (argc != 3) {
        printf("%s", help_message);
        return EXIT_SUCCESS;
    }

    char const *input_path = argv[1];
    char const *output_path = argv[2];

    FILE *f = fopen(input_path, "rb");
    if (!f) {
        fprintf(stderr, "Error: cannot open file '%s'\n", input_path);
        return EXIT_FAILURE;
    }

    struct str input = read_file(f);

    fclose(f);

    struct expr *expr = parse(input);
    print(expr);

    free_expr(expr);
    free(input.ptr);
}
