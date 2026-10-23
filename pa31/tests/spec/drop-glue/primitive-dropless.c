#include <stdio.h>
extern void observe(unsigned char*);int main(void){unsigned char v=7;observe(&v);printf("%u\n",(unsigned)v);return 0;}
