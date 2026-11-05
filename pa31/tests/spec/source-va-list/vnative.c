#include <stdio.h>
extern int trait_value(int);extern unsigned char va_value(unsigned char);int main(void){printf("%d %u\n",trait_value(23),(unsigned)va_value(17));return 0;}
