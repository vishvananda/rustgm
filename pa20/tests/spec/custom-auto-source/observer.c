#include <stdint.h>
#include <stdio.h>
extern uint8_t automatic_identity(uint8_t);
int main(void){uint8_t value=23;unsigned actual=automatic_identity(value);printf("auto %u\n",actual);return actual!=value;}
