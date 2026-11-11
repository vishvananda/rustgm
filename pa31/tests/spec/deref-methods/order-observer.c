#include <stdio.h>
static unsigned trace;
unsigned char tick(unsigned char code){trace=trace*10+code;return code==4?41:code;}
extern unsigned char observed_once(unsigned char);extern unsigned char observed_order(unsigned char);
int main(void){trace=0;unsigned char a=observed_once(23);unsigned x=trace;trace=0;unsigned char b=observed_order(23);unsigned y=trace;printf("receiver-order %u %u %u %u\n",(unsigned)a,x,(unsigned)b,y);return a!=23||x!=125||b!=41||y!=134625;}
