#include <stdint.h>
#include <stdio.h>
extern uint32_t observed_macro(void);
int main(void){uint32_t value=observed_macro();printf("%08x\n",value);return value==0x12345678u?0:1;}
