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
    const void* data;
}
Slice;

typedef struct State
{
    Mode mode;
    _Bool smart;

    Slice orig_text;
    Column orig_cursor_x;
    LineNumber orig_cursor_line;

    Slice input_lines;
    LineNumber input_line_no;
    Column input_x;

    LineNumber line_no;
    Slice ch;
    Column x;
    Column indent_x;
    _Bool return_parens;

    Column cursor_x;
    LineNumber cursor_line;
    Column prev_cursor_x;
}
State;

void state_init(State *state, const char* orig_text, size_t orig_text_length)
{
    state->orig_text.length = orig_text_length;
    state->orig_text.data = (void*)orig_text;
}

_Bool is_close_paren(const char* s)
{
    return *s && NULL != strchr(")]}", *s);
}

