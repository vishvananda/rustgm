#include <stdio.h>
static unsigned trace;
void record(unsigned char value){trace=trace*10u+value;}
extern unsigned char run(void);
extern unsigned char manual_needs(void),maybe_needs(void),lookalike_needs(void);
int main(void){unsigned value=run();printf("%u %u %u %u %u\n",trace,value,(unsigned)manual_needs(),(unsigned)maybe_needs(),(unsigned)lookalike_needs());}
