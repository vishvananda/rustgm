#include <stdio.h>

extern unsigned char observed_value(void);
extern unsigned char observed_empty(void);

int main(void) {
    printf("%u %u\n", observed_value(), observed_empty());
    return 0;
}
