#include <stdio.h>
extern unsigned char answer(void);
extern unsigned char second(void);
extern unsigned char runtime(void);
int main(void){printf("%u %u %u\n",(unsigned)answer(),(unsigned)second(),(unsigned)runtime());return 0;}
