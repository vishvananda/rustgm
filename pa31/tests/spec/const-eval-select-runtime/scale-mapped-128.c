#include <stdio.h>
static unsigned seen;
void record(unsigned char step){seen += step;}
extern unsigned char answer(void);
int main(void){unsigned char value=answer();printf("%u %u\n",(unsigned)value,seen);return 0;}
