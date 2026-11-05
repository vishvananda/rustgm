#include <stdio.h>
float float_value(void) __asm__("float");int main(void){printf("%.1f\n",(double)float_value());}
