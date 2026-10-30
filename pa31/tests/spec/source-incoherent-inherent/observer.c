#include <stdint.h>
#include <stdio.h>
extern uint8_t observed(void);
int main(void){printf("%u\n",(unsigned)observed());return 0;}
