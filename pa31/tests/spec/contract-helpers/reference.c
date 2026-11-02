#include <stdint.h>
#include <stdio.h>
extern uint8_t observed(void);
static uint8_t seen;
void record(uint8_t value){seen=value;}
int main(void){uint8_t result=observed();printf("%u %u\n",result,seen);return 0;}
