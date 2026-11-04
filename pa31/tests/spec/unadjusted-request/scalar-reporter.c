#include <stdio.h>
void report_values(unsigned char a,unsigned long b,unsigned __int128 c,float d,double e,_Bool f,unsigned long g,unsigned long h){printf("%u %lu %016lx%016lx %.2f %.2f %u %lu %lu\n",a,b,(unsigned long)(c>>64),(unsigned long)c,(double)d,e,(unsigned)f,g,h);}
