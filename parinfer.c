#include <stdlib.h>
#include <stdbool.h>
#include <string.h>

_Bool is_close_paren(const char* s)
{
    return *s && NULL != strchr(")]}", *s);
}

