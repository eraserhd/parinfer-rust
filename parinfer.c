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

typedef enum InTag
{
    IN_CODE,
    IN_COMMENT,
    IN_STRING,
    IN_LISP_READER_SYNTAX,
    IN_LISP_BLOCK_COMMENT_PRE,
    IN_LISP_BLOCK_COMMENT,
    IN_LISP_BLOCK_COMMENT_POST,
    IN_GUILE_BLOCK_COMMENT,
    IN_GUILE_BLOCK_COMMENT_POST,
    IN_JANET_LONG_STRING_PRE,
    IN_JANET_LONG_STRING,
}
InTag;

typedef union In
{
    InTag tag;
    struct {
        InTag tag;
        Slice delim;
    } string;
    struct {
        InTag tag;
        size_t depth;
    } depth;
    struct {
        InTag tag;
        size_t open_delim_len;
        size_t close_delim_len;
    } janet;
}
In;

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
    LineNumber prev_cursor_line;

    LineNumber selection_start_line;

    In context;
    Column comment_x;
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

