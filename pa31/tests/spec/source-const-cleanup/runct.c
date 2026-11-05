#include <stdio.h>
extern int run(int);
int main(void){int actual=run(59);printf("%d\n",actual);return actual!=59;}
