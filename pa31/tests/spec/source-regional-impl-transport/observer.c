#include <stdio.h>

extern unsigned char observed_bound(void);
extern unsigned char observed_method(void);

int main(void) {
    printf("%u %u\n", (unsigned)observed_bound(), (unsigned)observed_method());
    return 0;
}
