#include <stdint.h>
#include <stdio.h>
extern uint8_t add_value(uint8_t,const uint8_t*);
extern uint8_t sub_value(uint8_t,const uint8_t*);
int main(void){const uint8_t n=5;printf("%u %u\n",(unsigned)add_value(2,&n),(unsigned)sub_value(9,&n));return 0;}
