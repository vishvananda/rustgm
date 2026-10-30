#include <stdio.h>
extern unsigned runtime_default(void), runtime_required(void), const_value(void);
int main(void){unsigned a=runtime_default(),b=runtime_required(),c=const_value();printf("%u %u %u\n",a,b,c);return a!=7||b!=11||c!=13;}
