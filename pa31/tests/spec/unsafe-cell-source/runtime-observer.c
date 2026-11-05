#include <stdio.h>
#include <stdint.h>
uint32_t local_write(uint32_t);uint32_t global_write(uint32_t);int main(void){volatile uint32_t value=23;printf("%u %u\n",local_write(value),global_write(value));}
