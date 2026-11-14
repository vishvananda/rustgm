#include <stdio.h>
extern unsigned char entry(void);
int main(void){printf("%u\n",(unsigned)entry());return 0;}
