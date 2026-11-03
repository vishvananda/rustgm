#include <stdlib.h>
#include <stdio.h>
extern unsigned char run(void);
int main(void){printf("%u\n",(unsigned)run());return 0;}

void host_panic(const unsigned char *p,unsigned long n){(void)p;(void)n;exit(23);}
