#include <stdint.h>
#include <stdio.h>
extern void observed(void);static int called;void values(uint32_t a,uint32_t b,uint32_t c){called=a==5&&b==7&&c==11;printf("%u %u %u\n",a,b,c);}int main(void){observed();return called?0:1;}
