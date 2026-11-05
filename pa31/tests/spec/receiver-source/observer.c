#include <stdint.h>
#include <stdio.h>
uint32_t direct(void);uint32_t dot(void);uint32_t shared(void);uint32_t nested(void);uint32_t once(void);int main(void){printf("%u %u %u %u %u\n",direct(),dot(),shared(),nested(),once());}
