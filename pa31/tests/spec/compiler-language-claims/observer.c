#include <stdio.h>
extern unsigned char observed_move(void),observed_copy(void);
int main(void){printf("%u %u\n",(unsigned)observed_move(),(unsigned)observed_copy());return 0;}
