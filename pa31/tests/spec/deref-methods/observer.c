#include <stdio.h>
extern unsigned char observed_read(unsigned char);
extern unsigned char observed_mut(unsigned char);
int main(void){unsigned char a=observed_read(23),b=observed_mut(23);printf("receiver %u %u\n",(unsigned)a,(unsigned)b);return a!=23||b!=41;}
