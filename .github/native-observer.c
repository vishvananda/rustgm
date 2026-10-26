#include <stdbool.h>
#include <stdio.h>
extern bool rg_not_scalar(bool);
extern bool rg_not_overloaded(bool);
extern bool rg_not_constant(void);
int main(void) {
    const unsigned a = rg_not_scalar(false), b = rg_not_scalar(true);
    const unsigned c = rg_not_overloaded(false), d = rg_not_overloaded(true);
    const unsigned e = rg_not_constant();
    printf("%u %u %u %u %u\n", a, b, c, d, e);
    return a == 1 && b == 0 && c == 1 && d == 0 && e == 1 ? 0 : 1;
}
