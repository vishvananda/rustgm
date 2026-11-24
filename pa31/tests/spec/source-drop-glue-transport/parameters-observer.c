#include <stdint.h>
#include <stdio.h>
extern void observe_ordinary(uintptr_t);
extern void observe_glue(uintptr_t);
int main(void) {
    struct { unsigned char first; uint16_t second; unsigned char mark; } value = {7, 300, 41};
    observe_ordinary((uintptr_t)&value);
    printf("%u\n", value.mark);
    observe_glue((uintptr_t)&value);
    printf("%u\n", value.mark);
}
