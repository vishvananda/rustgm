#include <stdio.h>
static unsigned trace;
static unsigned before_value, after_value;
unsigned char tick(unsigned char code){trace=trace*10+code;return code==4?41:code;}
unsigned char observe(unsigned char before,unsigned char after){before_value=before;after_value=after;return before;}
extern unsigned char observed_coercion(unsigned char);
int main(void){unsigned char result=observed_coercion(23);printf("borrow-coercion %u %u %u\n",before_value,after_value,trace);return result!=23||before_value!=23||after_value!=41||trace!=1342;}
