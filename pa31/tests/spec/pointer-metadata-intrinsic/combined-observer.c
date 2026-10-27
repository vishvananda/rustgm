#include <stdio.h>
static unsigned calls;
void*mark(void){++calls;return 0;}
void observe(unsigned long a,unsigned long b,unsigned long c,unsigned long d,unsigned long e,unsigned long f){printf("%lu %lu %lu %lu %lu %lu %u\n",a,b,c,d,e,f,calls);}
#ifndef RUSTGM_CANDIDATE
extern void run(void);
int main(void){run();return 0;}
#endif
