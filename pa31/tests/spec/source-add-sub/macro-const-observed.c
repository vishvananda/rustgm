#include <stdint.h>
#include <stdio.h>
extern uint8_t observed_sum(void),observed_difference(void),observed_shared(void);
int main(void){printf("%u %u %u\n",(unsigned)observed_sum(),(unsigned)observed_difference(),(unsigned)observed_shared());return 0;}
