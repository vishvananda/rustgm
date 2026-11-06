#include <stdio.h>
void record(unsigned char value){printf("%u ",value);}
extern unsigned char observe(void);
int main(void){printf("%u\n",observe());return 0;}
