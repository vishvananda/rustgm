#include <stdarg.h>
#include <stdint.h>

const uint32_t SAFE_EXT = 23u;
uint32_t MUT_EXT = 29u;

int sum_va(int count, ...) {
    va_list args;
    va_start(args, count);
    int total = 0;
    for (int i = 0; i < count; ++i) {
        total += va_arg(args, int);
    }
    va_end(args);
    return total;
}
