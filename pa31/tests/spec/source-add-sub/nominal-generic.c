#include <stdint.h>
#include <stdio.h>
extern uint8_t overloaded(void),subtract(void);
int main(void){printf("%u %u\n",(unsigned)overloaded(),(unsigned)subtract());return 0;}
