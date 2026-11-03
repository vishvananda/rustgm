#include <stdlib.h>
#include <stdio.h>
extern unsigned char run(void);
static unsigned count;
void host_drop(unsigned char v){printf("drop %u\n",(unsigned)v);count++;}
int main(void){unsigned v=run();printf("value %u count %u\n",v,count);return 0;}

void host_panic(const unsigned char *p,unsigned long n){(void)p;(void)n;exit(23);}
