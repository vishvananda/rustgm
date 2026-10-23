#include <stdio.h>
struct D{unsigned char *p;};extern void observe(struct D**);int main(void){unsigned char count=0;struct D value={&count};struct D *borrowed=&value;observe(&borrowed);printf("%u\n",(unsigned)count);return 0;}
