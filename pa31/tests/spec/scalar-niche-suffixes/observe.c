#include <stdio.h>
extern unsigned char first(void);
extern unsigned char second(void);
int main(void) { printf("%u %u\n", (unsigned)first(), (unsigned)second()); return 0; }
