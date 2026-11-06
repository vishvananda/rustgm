#include <stdio.h>
extern unsigned char ready(void),pending(void),finished(void);
int main(void){printf("%u %u %u\n",ready(),pending(),finished());}
