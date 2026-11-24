#include <stdio.h>

unsigned char first(void);
unsigned char second(void);

int main(void) {
    printf("%u %u\n", (unsigned)first(), (unsigned)second());
    return 0;
}
