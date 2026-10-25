#include <stdio.h>
extern unsigned char constant(void),runtime(void),configured(void),generic_runtime(void);
int main(void){printf("%u %u %u %u\n",constant(),runtime(),configured(),generic_runtime());return 0;}
