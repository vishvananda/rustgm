#include <stdio.h>
extern unsigned char answer(unsigned char*,unsigned char*);int main(void){unsigned char first=3,second=7;unsigned char value=answer(&first,&second);printf("%u %u %u\n",(unsigned)value,(unsigned)first,(unsigned)second);return value!=9||first!=3||second!=9;}
