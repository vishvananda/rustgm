#include <stdio.h>
extern unsigned char selected(void);extern unsigned char ordinary(void);int main(void){printf("%u\n%u\n",selected(),ordinary());return 0;}
