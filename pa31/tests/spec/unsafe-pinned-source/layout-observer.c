#include <stdio.h>
#include <stddef.h>
size_t anchor_size(void);size_t anchor_option_size(void);size_t slot_option_size(void);size_t plain_option_size(void);int main(void){printf("%zu %zu %zu %zu\n",anchor_size(),anchor_option_size(),slot_option_size(),plain_option_size());}
