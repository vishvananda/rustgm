#include <stdio.h>
#include <stddef.h>
extern size_t str_length(void),slice_length(void),const_length(void);
int main(void){printf("%zu %zu %zu\n",str_length(),slice_length(),const_length());return 0;}
