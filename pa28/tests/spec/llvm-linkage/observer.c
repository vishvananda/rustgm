#include <stdio.h>
unsigned long long word(unsigned char i){const unsigned long long words[4]={0,~0ULL,0x81b70ceff00d1513ULL,0x0123456789abcdefULL};return words[i];}
void observe(unsigned char x){printf("%u\n",(unsigned)x);}
