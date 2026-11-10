#include <stdio.h>
extern unsigned char constant(void);
extern unsigned char runtime(void);
extern unsigned char arguments(void);
int main(void){printf("%u %u %u\n",constant(),runtime(),arguments());return 0;}
