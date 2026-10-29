#include <stdint.h>
#include <stdio.h>
extern uint8_t observed(void);
int main(void){uint8_t value=observed();printf("%u\n",(unsigned)value);return value==7?0:1;}
