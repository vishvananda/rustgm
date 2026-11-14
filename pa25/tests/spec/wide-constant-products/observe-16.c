#include <stdio.h>
#ifndef COUNT
#define COUNT 18
#endif
#include <stdint.h>
#include <string.h>
extern const unsigned char* address(void);int main(void){const unsigned char*p=address();for(unsigned i=0;i<COUNT;i++){uint16_t x;memcpy(&x,p+2*i,2);if((x&0x7c00)==0x7c00&&(x&0x3ff))puts("nan");else printf("%04x\n",(unsigned)x);}return 0;}
