#include <stdio.h>
int builtin(int,int);int explicit(int,int);int main(void){volatile int l=8,r=3;printf("%d %d\n",builtin(l,r),explicit(l,r));return 0;}
