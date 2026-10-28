#include <stdio.h>
extern unsigned char answer(void);void seen(unsigned char s,unsigned char v){printf("%u:%u;",s,v);}int main(void){printf("%u\n",answer());return 0;}
