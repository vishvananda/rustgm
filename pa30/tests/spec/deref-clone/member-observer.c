#include <stdint.h>
#include <stdio.h>
static unsigned trace;
void mark(uint8_t step){trace=trace*10+step;}
extern uint8_t code(void);
int main(void){unsigned value=code();printf("deref-member %u %u\n",value,trace);return value==23&&trace==12?0:1;}
