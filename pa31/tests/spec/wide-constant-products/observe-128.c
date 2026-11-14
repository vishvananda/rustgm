#include <stdio.h>
#ifndef COUNT
#define COUNT 18
#endif
#include <stdint.h>
#include <string.h>
extern const unsigned char* address(void);int main(void){const unsigned char*p=address();for(unsigned i=0;i<COUNT;i++){uint64_t lo,hi;memcpy(&lo,p+16*i,8);memcpy(&hi,p+16*i+8,8);if((hi&0x7fff000000000000ULL)==0x7fff000000000000ULL&&((hi&0xffffffffffffULL)||lo))puts("nan");else printf("%016llx%016llx\n",(unsigned long long)hi,(unsigned long long)lo);}return 0;}
