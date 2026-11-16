#include <stdint.h>
#include <stdio.h>
extern void observed(void);
void values(uint8_t a, uint8_t b, uint8_t c) { printf("%u %u %u\n", a, b, c); }
int main(void) { observed(); return 0; }
