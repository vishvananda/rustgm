#include <stdint.h>
#include <stdio.h>
extern uint8_t constant(void),runtime(void),inherited_runtime(void);
int main(void){printf("%u %u %u\n",constant(),runtime(),inherited_runtime());return 0;}
