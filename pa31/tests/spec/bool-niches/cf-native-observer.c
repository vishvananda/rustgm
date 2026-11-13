#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>
extern uint8_t encode(bool,bool);extern uint8_t decode(uint8_t);
int main(void){printf("%u %u %u %u %u %u\n",encode(false,false),encode(true,false),encode(false,true),decode(0),decode(1),decode(2));return 0;}
