#include <stdio.h>
void observe(unsigned char a, unsigned char b, unsigned char c, unsigned char d,
             unsigned char e, unsigned char f, unsigned char g) {
    printf("%u %u %u %u %u %u %u\n", (unsigned)a, (unsigned)b, (unsigned)c,
           (unsigned)d, (unsigned)e, (unsigned)f, (unsigned)g);
}
#ifndef RUSTGM_CANDIDATE
extern void run(void);
int main(void) { run(); }
#endif
