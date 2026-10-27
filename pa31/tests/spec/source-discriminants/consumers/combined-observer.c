#include <stdint.h>
#include <stdio.h>

void observe(int8_t a, uint16_t b, intptr_t c, intptr_t d, uint8_t e) {
    printf("%d %u %ld %ld %u\n", (int)a, (unsigned)b,
           (long)c, (long)d, (unsigned)e);
}
#ifndef RUSTGM_CANDIDATE
extern void run(void);
int main(void) { run(); return 0; }
#endif
