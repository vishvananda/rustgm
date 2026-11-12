#include <stdint.h>
#include <stdio.h>
static unsigned trace;static uint8_t values[5];static unsigned count;
void record(uint8_t tag,uint8_t value){trace=trace*10+tag;if(count<5)values[count]=value;count++;}
extern uint8_t code(uint8_t value);
int main(void){uint8_t result=code(7);printf("match-capture %u %u %u\n",values[2],result,trace);return count==5&&values[0]==1&&values[1]==7&&values[2]==7&&values[3]==0&&values[4]==9&&result==9&&trace==12314?0:1;}
