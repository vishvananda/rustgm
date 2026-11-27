#include <stdio.h>
extern unsigned char observe(void);
int main(void){unsigned int value=observe();printf("%u\n",value);return value==3?0:1;}
