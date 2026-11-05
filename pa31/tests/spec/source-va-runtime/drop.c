#include <stdio.h>
extern int va_order(int*,...);extern int va_return(int*,...);int main(void){int a=0,b=0;int x=va_order(&a,41,43);int y=va_return(&b,47);printf("%d %d %d %d\n",x,a,y,b);return 0;}
