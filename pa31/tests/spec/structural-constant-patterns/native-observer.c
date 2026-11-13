#include <stdio.h>
extern unsigned char alignment(unsigned char),structural(unsigned char),inactive(unsigned char);int main(void){printf("%u %u %u %u %u %u %u\n",alignment(0),alignment(1),structural(7),structural(8),inactive(0),inactive(1),inactive(2));}
