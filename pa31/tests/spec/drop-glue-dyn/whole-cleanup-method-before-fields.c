#include <stdio.h>
static unsigned count=0;void record(unsigned char n){count=count*10+n;}extern void observe(void*);int main(void){unsigned char data[2]={0};observe(&data[0]);printf("%u\n",count);return 0;}
