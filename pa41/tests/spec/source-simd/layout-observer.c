#include <stdint.h>
#include <stdio.h>
extern uint64_t layout_bits(void),aligned(void);
int main(void){uint64_t a=layout_bits(),b=aligned();for(unsigned i=0;i<8;++i)printf("%u ",(unsigned)((a>>(8*i))&255));printf("%u %u\n",(unsigned)(b&255),(unsigned)((b>>8)&255));return 0;}
