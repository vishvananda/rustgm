#include <stdio.h>
extern unsigned long syntax(unsigned long);
extern unsigned long explicit_shift(unsigned long);
extern unsigned long explicit_or(unsigned long);
int main(void){printf("%lu %lu %lu\n",syntax(3),explicit_shift(3),explicit_or(3));return 0;}
