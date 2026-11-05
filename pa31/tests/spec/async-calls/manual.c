#include <stdint.h>
#include <stdio.h>
uint32_t once(void),shared(void),mutable(void),forwarded(void),unique(void);int main(void){printf("%u %u %u %u %u\n",once(),shared(),mutable(),forwarded(),unique());}
