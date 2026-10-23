#include <stdio.h>
struct D{unsigned char *p;};extern void observe(struct D*);int main(void){unsigned char count=0;struct D value={&count};observe(&value);printf("%u\n",(unsigned)count);return 0;}
