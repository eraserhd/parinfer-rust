#include <stdlib.h>
#include <stdbool.h>
#include <string.h>

typedef size_t Column;
typedef size_t LineNumber;

#define NO_COLUMN ((size_t)-1)
#define NO_LINE_NUMBER ((size_t)-1)

typedef enum Mode
{
    MODE_INDENT,
    MODE_PAREN
}
Mode;

typedef struct Slice
{
    size_t length;
    void* data;
}
Slice;

void slice_destroy(Slice* slice)
{
    if (NULL != slice->data)
        free(slice->data);
    slice->data = NULL;
    slice->length = 0;
}

typedef struct State
{
    Mode mode;
    _Bool smart;

    Slice orig_text;
    Column orig_cursor_x;
    LineNumber orig_cursor_line;

    LineNumber input_line_count;
}
State;

void state_init(State *state, const char* orig_text)
{
    state->orig_text.length = strlen(orig_text);
    state->orig_text.data = (void*)strdup(orig_text);
}

void state_destroy(State *state)
{
    slice_destroy(&state->orig_text);
}

_Bool is_close_paren(const char* s)
{
    return *s && NULL != strchr(")]}", *s);
}

