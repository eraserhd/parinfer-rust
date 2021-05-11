#include <stdlib.h>
#include <stdbool.h>
#include <string.h>

typedef enum Mode
{
    MODE_INDENT,
    MODE_PAREN
}
Mode;

typedef struct State
{
    Mode mode;
}
State;

_Bool is_close_paren(const char* s)
{
    return *s && NULL != strchr(")]}", *s);
}

