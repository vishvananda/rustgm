#include <stdio.h>
int index_array_key(int,int);int index_array_write(int,int,int);int index_array_u8(int,int);int main(void){volatile int left=11,right=17,value=23;printf("%d %d %d\n",index_array_key(left,right),index_array_write(left,right,value),index_array_u8(left,right));return 0;}
