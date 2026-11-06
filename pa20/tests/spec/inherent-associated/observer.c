#include <stdint.h>
#include <stdio.h>
extern uint8_t alias_identity(uint8_t);
int main(void){unsigned result=alias_identity(23);printf("alias %u\n",result);return result!=23;}
