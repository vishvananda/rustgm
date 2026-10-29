#include <stdint.h>
#include <stdio.h>
extern uint8_t answer(void);int main(void){uint8_t x=answer();printf("%u\n",x);return x==7?0:1;}
