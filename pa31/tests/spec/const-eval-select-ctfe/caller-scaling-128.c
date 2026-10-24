#include <stdio.h>
extern unsigned char answer(void);
extern unsigned char runtime(void);
int main(void){printf("%u %u\n",(unsigned)answer(),(unsigned)runtime());return 0;}
