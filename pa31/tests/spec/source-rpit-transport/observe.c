#include <stdint.h>
#include <stdio.h>

extern void observe_rpit(
    uint32_t *a,
    uint32_t *b,
    uint32_t *c,
    uint32_t *d,
    uint32_t *e
);

int main(void) {
    uint32_t a = 0, b = 0, c = 0, d = 0, e = 0;
    observe_rpit(&a, &b, &c, &d, &e);
    printf("%u %u %u %u %u\n", a, b, c, d, e);
    return 0;
}
