#include <stdint.h>
#include <stdio.h>
static unsigned calls;
uint8_t cold_external(uint8_t value){++calls;return (uint8_t)(value+5);}
uint8_t cold_safe(uint8_t value){++calls;return (uint8_t)(value+7);}
extern uint8_t observed(uint8_t);extern uint8_t observed_safe(uint8_t);
int main(void){unsigned a=observed(23),b=observed_safe(29);printf("cold-foreign %u %u %u\n",a,b,calls);return !(a==28 && b==36 && calls==2);}
