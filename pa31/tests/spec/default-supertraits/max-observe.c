#include <stdio.h>
extern unsigned char entry(void);extern unsigned char probe(unsigned char,unsigned char);int main(void){printf("%u %u\n",(unsigned)entry(),(unsigned)probe(7,41));}
