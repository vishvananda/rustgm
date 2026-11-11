#include <stdio.h>
static unsigned trace, after_value;
unsigned char tick(unsigned char code){trace=trace*10+code;return code==4?41:code;}
void observe_drop(unsigned char n){after_value=n;}
extern unsigned char observed_guard(unsigned char);
int main(void){unsigned char before=observed_guard(23);printf("borrow-guard %u %u %u\n",(unsigned)before,after_value,trace);return before!=23||after_value!=41||trace!=1345;}
