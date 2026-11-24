#include <stdio.h>

extern unsigned char observed(unsigned char);

int main(void) {
    printf("%u\n", observed(73));
    return 0;
}
