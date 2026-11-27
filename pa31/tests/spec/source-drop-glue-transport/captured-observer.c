#include <stdint.h>
#include <stdio.h>

extern void bounded(uintptr_t);
extern void conditional(uintptr_t);

struct bounded {
    const unsigned char *input;
    const unsigned char *other;
    unsigned char marks[3];
    unsigned char state;
};

struct conditional {
    const unsigned char *input;
    unsigned char state;
};

int main(void) {
    const unsigned char input = 11;
    struct bounded a = { &input, &input, {1, 2, 3}, 0 };
    struct conditional b = { &input, 0 };
    bounded((uintptr_t)&a);
    conditional((uintptr_t)&b);
    printf("%u %u\n", (unsigned)a.state, (unsigned)b.state);
    return 0;
}
