#include <stdio.h>
extern unsigned char observed(unsigned char);
int main(void){unsigned char value=observed(23);printf("adapter %u\n",(unsigned)value);return value!=23;}
