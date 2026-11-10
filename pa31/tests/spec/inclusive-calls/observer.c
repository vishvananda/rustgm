#include <stdio.h>
extern unsigned char observed(void);void record(unsigned char v){printf("%u ",v);}int main(void){unsigned char v=observed();printf("%u\n",v);}
