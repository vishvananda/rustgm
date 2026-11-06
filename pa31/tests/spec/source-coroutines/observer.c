#include <stdio.h>
extern unsigned char first(void);
extern unsigned char complete(void);
int main(void) { printf("%u %u\n", first(), complete()); return 0; }
