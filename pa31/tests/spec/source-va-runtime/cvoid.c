#include <stdio.h>
extern unsigned char cv_echo(unsigned char);int main(void){printf("%u\n",(unsigned)cv_echo(1));return 0;}
