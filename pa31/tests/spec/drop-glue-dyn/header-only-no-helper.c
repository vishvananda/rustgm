#include <stdio.h>
extern unsigned char observe(void*);int main(void){unsigned char value=0;printf("%u\n",(unsigned)observe(&value));return 0;}
