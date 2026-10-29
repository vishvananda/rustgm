#include <stdint.h>
#include <stdio.h>
extern uint32_t observed(void);
int main(void){uint32_t x=observed();printf("%u\n",x);return x!=7;}
