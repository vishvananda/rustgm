#include <stdint.h>
#include <stdio.h>
extern uint8_t native_provider(void);
extern uint8_t consumer_observed(void);
int main(void){printf("%u %u\n",(unsigned)native_provider(),(unsigned)consumer_observed());return 0;}
