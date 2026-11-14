#include <stdio.h>
#include <stddef.h>
extern size_t pointer_words_size(void);int main(void){printf("%zu\n",pointer_words_size());return 0;}
