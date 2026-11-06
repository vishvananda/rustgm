#include <stdio.h>
extern unsigned char custom(unsigned char,unsigned char*);extern unsigned char blanket(unsigned char);extern unsigned char borrowed(unsigned char);
int main(void){unsigned char out=0;unsigned char got=custom(23,&out);printf("%u %u %u %u\n",got,out,blanket(47),borrowed(59));}
