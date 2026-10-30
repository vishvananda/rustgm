#include <stdint.h>
#include <stdio.h>
extern uint8_t observed(void);
extern uint8_t second(void);
int main(void){printf("%u %u\n",(unsigned)observed(),(unsigned)second());return 0;}
