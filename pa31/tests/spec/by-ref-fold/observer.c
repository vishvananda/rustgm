#include <stdint.h>
#include <stdio.h>
static unsigned trace;
void mark(uint8_t step){trace=trace*10+step;}
uint8_t combine(uint8_t a,uint8_t b){mark(4);return a==11&&b==12?a+b:0;}
extern uint8_t code(void);
int main(void){uint8_t value=code();printf("by-ref-fold %u %u\n",(unsigned)value,trace);return value==23&&trace==1234?0:1;}
