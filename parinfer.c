#include <stdlib.h>
#include <stdbool.h>
#include <string.h>

typedef size_t Column;

#define NO_COLUMN ((size_t)-1)

typedef enum Mode
{
    MODE_INDENT,
    MODE_PAREN
}
Mode;

typedef struct State
{
    Mode mode;
    _Bool smart;

    char* orig_text;
    Column orig_cursor_x;
}
State;

void state_init(State *state, const char* orig_text)
{
    state->orig_text = strdup(orig_text);
}

void state_destroy(State *state)
{
    if (NULL == state->orig_text)
        free(state->orig_text);
    state->orig_text = NULL;
}

_Bool is_close_paren(const char* s)
{
    return *s && NULL != strchr(")]}", *s);
}

