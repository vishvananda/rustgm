#include <stdio.h>
#include <stdint.h>
typedef struct{uint8_t item,value;}Pair;extern Pair pair(Pair);extern uint8_t indirect(uint8_t,uint8_t(*)(uint8_t));extern uint8_t imported(uint8_t);extern uint16_t generic(uint16_t);uint8_t callback(uint8_t value){return value;}int main(void){Pair p=pair((Pair){1,73});printf("%u %u %u %u %u\n",p.item,p.value,indirect(1,callback),imported(1),generic(513));return 0;}
