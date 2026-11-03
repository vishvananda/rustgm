#include <stdio.h>
extern unsigned char observed_kind(void);
int main(void){printf("%u\n",(unsigned)observed_kind());return 0;}
