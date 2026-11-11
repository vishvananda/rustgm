#include <stdio.h>
extern unsigned char observed_const(void);
int main(void){unsigned char v=observed_const();printf("borrow-const %u\n",(unsigned)v);return v!=41;}
