#include <stdio.h>
#include <stdlib.h>
extern unsigned char run(void);
void host_panic(const unsigned char *p,unsigned long n){fwrite(p,1,n,stdout);putchar(10);exit(23);}
int main(void){return run();}
