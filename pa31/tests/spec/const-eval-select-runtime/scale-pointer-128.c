#include <stdio.h>
static unsigned seen;
void record(unsigned char step){seen += step;}
extern unsigned char answer(_Bool);
int main(void){unsigned char a=answer(1);unsigned first=seen;seen=0;unsigned char b=answer(0);printf("%u %u %u %u\n",(unsigned)a,first,(unsigned)b,seen);return 0;}
