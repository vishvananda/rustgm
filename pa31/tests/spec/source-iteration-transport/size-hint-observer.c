#include <stdio.h>

extern unsigned char observed_runtime(void);
extern unsigned char observed_const(void);

int main(void) {
    printf("%u %u\n", observed_runtime(), observed_const());
    return 0;
}
