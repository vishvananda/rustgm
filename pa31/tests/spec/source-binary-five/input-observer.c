#include <stdio.h>
int observe_runtime(int,unsigned char);
int main(void){volatile int left=22;volatile unsigned char right=23;printf("%d\n",observe_runtime(left,right));return 0;}
