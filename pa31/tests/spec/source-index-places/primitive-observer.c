#include <stdio.h>
int index_builtin(int,int);int index_explicit(int,int);int main(void){volatile int left=11,right=17;printf("%d %d\n",index_builtin(left,right),index_explicit(left,right));return 0;}
