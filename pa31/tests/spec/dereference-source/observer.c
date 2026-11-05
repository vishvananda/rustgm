#include <stdio.h>
void record(unsigned char n){printf("record=%u\n",(unsigned)n);}
extern unsigned char read(void),write(void),generic(void),once(void);
int main(void){unsigned char a=read();printf("read=%u\n",(unsigned)a);a=write();printf("write=%u\n",(unsigned)a);a=generic();printf("generic=%u\n",(unsigned)a);a=once();printf("once=%u\n",(unsigned)a);return 0;}
