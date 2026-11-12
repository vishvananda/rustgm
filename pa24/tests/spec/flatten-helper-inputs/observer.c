#include <stdint.h>
#include <stdio.h>
static unsigned trace,count;static uint8_t values[4];
void record(uint8_t tag,uint8_t value){trace=trace*10+tag;if(count<4)values[count]=value;count++;}
extern uint8_t code(uint8_t value);
int main(void){uint8_t value=code(7);printf("flatten-helper %u %u %u\n",values[0],value,trace);return value==9&&count==4&&trace==1234&&values[0]==7&&values[1]==7&&values[2]==9&&values[3]==9?0:1;}
