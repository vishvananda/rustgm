#include <stdint.h>
#include <stdio.h>
static unsigned trace;static uint8_t values[3];
void record(uint8_t tag,uint8_t value){trace=trace*10+tag;if(tag>=1&&tag<=3)values[tag-1]=value;}
extern uint8_t code(uint8_t value);
int main(void){uint8_t value=code(7);printf("try-capture %u %u %u %u\n",values[0],values[1],values[2],trace);return value==23&&values[0]==7&&values[1]==43&&values[2]==23&&trace==123?0:1;}
