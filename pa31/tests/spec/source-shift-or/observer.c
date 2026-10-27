#include <stdio.h>
extern unsigned long encoded(unsigned long);
extern unsigned long shifted(void);
extern unsigned long ored(void);
int main(void){printf("%lu %lu %lu\n",encoded(3),shifted(),ored());return 0;}
