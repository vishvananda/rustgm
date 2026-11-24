#include <stdio.h>

extern unsigned char observed_runtime(void);
extern unsigned char observed_const(void);
extern unsigned char observed_empty(void);

int main(void) {
    printf("%u %u %u\n", observed_runtime(), observed_const(), observed_empty());
    return 0;
}
