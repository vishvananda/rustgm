#include <stdio.h>
extern unsigned char answer(void);
int main(void) { unsigned char value = answer(); printf("%u\n", (unsigned)value); return value == 7 ? 0 : 1; }
