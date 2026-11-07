#include <stdint.h>
#include <stdio.h>
extern uint8_t observed(const uint8_t *,const uint8_t *);extern uint8_t observed_const(void);
int main(void){uint8_t a=17,b=23;printf("guaranteed %u %u %u\n",(unsigned)observed(&a,&a),(unsigned)observed(&a,&b),(unsigned)observed_const());return 0;}
