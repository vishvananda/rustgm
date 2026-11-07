#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>
extern uint8_t observed(uint8_t,bool,uint8_t);
int main(void){unsigned a=observed(23,true,71),b=observed(23,false,71);printf("cold %u %u\n",a,b);return a==23&&b==71?0:1;}
