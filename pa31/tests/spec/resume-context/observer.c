#include <stdio.h>
extern unsigned char context_value(unsigned char);
int main(void){printf("%u %u\n",context_value(23),context_value(47));}
