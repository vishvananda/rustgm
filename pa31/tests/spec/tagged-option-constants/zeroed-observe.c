#include <stdio.h>
extern unsigned char zeroed(void);
extern unsigned char ones(void);
int main(void){printf("zeroed %u\nones %u\n",(unsigned)zeroed(),(unsigned)ones());return 0;}
