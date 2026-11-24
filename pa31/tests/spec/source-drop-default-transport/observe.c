#include <stdint.h>
#include <stdio.h>
extern void observe_ordinary(uintptr_t);
extern void observe_glue(uintptr_t);
int main(void) {
    unsigned char local = 41;
    struct { unsigned char mark; } provider = {41};
    observe_ordinary((uintptr_t)&local);
    printf("%u\n", (unsigned)local);
    observe_glue((uintptr_t)&provider);
    printf("%u\n", (unsigned)provider.mark);
}
