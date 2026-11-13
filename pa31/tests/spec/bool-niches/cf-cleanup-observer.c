#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>
static unsigned hits,trace;
void observed(uint8_t value){++hits;trace=10*trace+value;}
extern void cleanup(bool,bool);
int main(void){cleanup(true,false);printf("%u:%u ",hits,trace);cleanup(false,false);printf("%u:%u ",hits,trace);cleanup(false,true);printf("%u:%u ",hits,trace);cleanup(true,true);printf("%u:%u\n",hits,trace);return 0;}
