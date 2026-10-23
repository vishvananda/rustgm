#include <stdio.h>
extern unsigned char observe(const unsigned char*);int main(void){unsigned char x=7;printf("%u\n",(unsigned)observe(&x));return 0;}
