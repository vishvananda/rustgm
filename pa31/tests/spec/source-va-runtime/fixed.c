#include <stdio.h>
extern double va_fixed(double,...);int main(void){printf("%.1f\n",va_fixed(10.0,19.5,31.5));return 0;}
