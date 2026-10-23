#include <stdio.h>
static unsigned steps=0;void record(unsigned char step){steps=steps*10+step;}extern unsigned char answer(void);int main(void){unsigned char result=answer();printf("%u %u\n",result,steps);return 0;}
