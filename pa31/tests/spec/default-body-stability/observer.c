#include <stdio.h>
extern unsigned char observed_default(void),observed_override(void);
int main(void){printf("%u %u\n",(unsigned)observed_default(),(unsigned)observed_override());return 0;}
