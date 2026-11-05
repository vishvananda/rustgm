#include <stdio.h>
#include <stddef.h>
size_t index_length(void);unsigned char index_unsized(unsigned char);int main(void){volatile unsigned char value=23;printf("%zu %u\n",index_length(),index_unsized(value));return 0;}
