#include <stdint.h>
#include <stdio.h>
extern void observed(void);static int called;void values(uint8_t a,uint8_t b,uint8_t c){called=a==7&&b==8&&c==9;printf("%u %u %u\n",a,b,c);}int main(void){observed();return called?0:1;}
