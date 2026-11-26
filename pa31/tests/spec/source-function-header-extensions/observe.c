#include <stdint.h>
#include <stdio.h>

extern void observe_extensions(
    uint32_t *a,
    uint32_t *b,
    uint32_t *c,
    uint32_t *d,
    uint32_t *e,
    uint32_t *f,
    uint32_t *g,
    uint32_t *h
);

int main(void) {
    uint32_t a = 0, b = 0, c = 0, d = 0, e = 0;
    uint32_t f = 0, g = 0, h = 0;
    observe_extensions(&a, &b, &c, &d, &e, &f, &g, &h);
    printf("%u %u %u %u %u %u %u %u\n", a, b, c, d, e, f, g, h);
    return 0;
}
