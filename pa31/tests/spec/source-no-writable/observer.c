#include <stdint.h>
#include <stdio.h>
extern uint32_t observed_writable(void);
int main(void){uint32_t x=observed_writable();printf("%u\n",x);return x==7?0:1;}
