#include <stdio.h>
void record(unsigned char x){printf("record=%u\n",(unsigned)x);}
extern unsigned char scalar(void),borrowed(void),owned(void);
int main(void){unsigned char a=scalar();printf("scalar=%u\n",(unsigned)a);unsigned char b=borrowed();printf("borrowed=%u\n",(unsigned)b);unsigned char c=owned();printf("owned=%u\n",(unsigned)c);return 0;}
