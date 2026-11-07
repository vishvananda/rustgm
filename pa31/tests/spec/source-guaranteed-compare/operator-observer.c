#include <stdio.h>
extern unsigned char observed(const unsigned char*,const unsigned char*);
extern unsigned char explicit(const unsigned char*,const unsigned char*);
int main(void){unsigned char a=17,b=23;printf("primitive-pointer %u %u %u %u\n",observed(&a,&a),observed(&a,&b),explicit(&a,&a),explicit(&a,&b));return 0;}
