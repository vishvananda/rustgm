#include <stdio.h>
extern unsigned char fwref(unsigned char);extern unsigned char fwpin(unsigned char);
int main(void){printf("%u %u\n",fwref(71),fwpin(83));}
