#include <stdint.h>
#include <stdio.h>
uint32_t generic_once(void),generic_shared(void),generic_mutable(void);int main(void){printf("%u %u %u\n",generic_once(),generic_shared(),generic_mutable());}
