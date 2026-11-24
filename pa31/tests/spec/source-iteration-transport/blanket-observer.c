#include <stdio.h>

extern unsigned char observed_value(void);
extern unsigned char observed_empty(void);

extern unsigned char observed_const(void);

int main(void) {
    printf("%u %u %u\n", observed_value(), observed_empty(), observed_const());
    return 0;
}
