#include <stdio.h>

extern unsigned char observed_runtime(unsigned char);
extern unsigned char observed_const(void);

int main(void) {
    printf("%u %u\n", observed_runtime(73), observed_const());
    return 0;
}
