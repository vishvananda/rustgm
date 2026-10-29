#include <stdint.h>
#include <stdio.h>
extern uint8_t original(void),explicit(uint8_t),implicit(uint8_t),explicit_unsafe(uint8_t),safe_block(uint8_t),constant(void);
int main(void){printf("%u %u %u %u %u %u\n",original(),explicit(17),implicit(23),explicit_unsafe(29),safe_block(31),constant());return 0;}
