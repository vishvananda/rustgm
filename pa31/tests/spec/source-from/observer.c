#include <stdio.h>
extern unsigned char observed(void);
int main(void){printf("%u\n",(unsigned)observed());return 0;}
