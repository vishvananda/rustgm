#include <stdint.h>
#include <stdio.h>

extern void observe_ordinary(uintptr_t);
extern void observe_glue(uintptr_t);

int main(void) {
    uint8_t value = 41;
    observe_ordinary((uintptr_t)&value);
    printf("%u\n", value);
    observe_glue((uintptr_t)&value);
    printf("%u\n", value);
}
