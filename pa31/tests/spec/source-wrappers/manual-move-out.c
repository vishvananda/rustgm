#include <stdio.h>
static unsigned trace;
void record(unsigned char value){trace=trace*10u+value;}
extern unsigned char run(void);
int main(void){unsigned value=run();printf("%u %u\n",trace,value);}
