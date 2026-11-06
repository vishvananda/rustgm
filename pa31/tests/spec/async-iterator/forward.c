#include <stdio.h>
extern unsigned char borrowed(void),pinned(void);
int main(void){printf("%u %u\n",borrowed(),pinned());}
