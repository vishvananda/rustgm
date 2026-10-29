#include <stdint.h>
#include <stdio.h>
extern uint8_t blanket_value(void);
int main(void){unsigned value=blanket_value();printf("%u\n",value);return value!=7;}
