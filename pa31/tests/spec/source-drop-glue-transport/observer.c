#include <stdint.h>
#include <stdio.h>

typedef struct D { unsigned char mark; } D;
extern void observe_ordinary(uintptr_t);
extern void observe_glue(uintptr_t);
static unsigned count, sum;

void record_drop(unsigned char mark) {
    ++count;
    sum += mark;
}

int main(void) {
    D value = {41};
    observe_ordinary((uintptr_t)&value);
    printf("%u %u\n", count, sum);
    observe_glue((uintptr_t)&value);
    printf("%u %u\n", count, sum);
}
