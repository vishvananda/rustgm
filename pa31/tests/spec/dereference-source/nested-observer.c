#include <stdio.h>
extern unsigned char nested_borrow(void);
int main(void){printf("%u\n",(unsigned)nested_borrow());}
