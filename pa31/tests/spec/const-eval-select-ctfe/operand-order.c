#include <stdio.h>
extern unsigned char answer(void);
extern unsigned char order(void);
extern unsigned char runtime(void);
extern unsigned char runtime_order(void);
int main(void){printf("%u %u %u %u\n",(unsigned)answer(),(unsigned)order(),(unsigned)runtime(),(unsigned)runtime_order());return 0;}
