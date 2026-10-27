#include <stdint.h>
#include <stdio.h>
extern uint8_t explicit_add(uint8_t,uint8_t),explicit_sub(uint8_t,uint8_t);
extern uint8_t shared_add(uint8_t,const uint8_t*),shared_sub(uint8_t,const uint8_t*);
int main(void){const uint8_t n=2;printf("%u %u %u %u\n",(unsigned)explicit_add(9,2),(unsigned)explicit_sub(9,2),(unsigned)shared_add(9,&n),(unsigned)shared_sub(9,&n));return 0;}
