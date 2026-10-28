#include <stdint.h>
#include <stdio.h>
extern uint32_t answer(void);
void cloned(uint8_t value){printf("c%u;",value);}
void observed(uint8_t value){printf("v%u;",value);}
int main(void){uint32_t value=answer();printf("%u\n",value);return value==7?0:1;}
