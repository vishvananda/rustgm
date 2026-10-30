#include <stdint.h>
#include <stdio.h>
extern uint32_t observed(void);int main(void){uint32_t n=observed();printf("%u\n",n);return n!=7;}
