#include <stdio.h>
extern unsigned char pin_forward(unsigned char),ref_forward(unsigned char);
int main(void){printf("%u %u\n",(unsigned)pin_forward(23),(unsigned)ref_forward(47));return 0;}
