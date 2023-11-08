#include <stdarg.h>
#include <stdio.h>

void udi_debug_printf(const char* format, ...)
{
    va_list args;
    va_start(args, format);
    printf("udi_debug_printf: ");
    vprintf(format, args);
    va_end(args);
    printf("\n");
}