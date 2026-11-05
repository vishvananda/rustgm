#include <stdio.h>
extern void consume(unsigned char);static int log;void seen(unsigned char value){log=value;}int main(void){consume(1);printf("%d\n",log);return 0;}
