#include <stdio.h>
static unsigned visits;
unsigned char mark(void){visits++;return (unsigned char)visits;}
extern unsigned char multi_concrete(unsigned char*,unsigned char),multi_explicit(unsigned char*,unsigned char),projected(unsigned char*,unsigned char);
int main(void){unsigned char a=1,b=2,d=3;unsigned x=multi_concrete(&a,23),y=multi_explicit(&b,31);if(visits)return 1;unsigned p=projected(&d,59);if(x!=17||a!=23||y!=19||b!=31||p!=23||d!=59||visits!=1)return 2;printf("successor %u %u %u %u %u %u %u\n",x,a,y,b,p,d,visits);return 0;}
