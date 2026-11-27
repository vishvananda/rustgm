#include <stdio.h>
void record(unsigned char n){printf("record=%u\n",(unsigned)n);}
extern unsigned char run(void);int main(void){unsigned char n=run();printf("result=%u\n",(unsigned)n);return 0;}
