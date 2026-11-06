#include <stdio.h>
extern unsigned char field_constant(void);
int main(void){printf("const=%u\n",(unsigned)field_constant());}
