#include <stdint.h>
#include <stdio.h>
extern uint8_t entry(void);
extern uint8_t probe(uint8_t);
extern uint8_t fail(void);
int main(void){printf("%u %u %u\n",entry(),probe(59),fail());return 0;}
