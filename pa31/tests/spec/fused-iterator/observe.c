#include <stdio.h>
extern unsigned char first(void), second(void), third(void);
int main(void) {printf("%u %u %u\n", first(), second(), third()); return 0;}
