#include <stdio.h>
#include <stddef.h>
extern size_t option_u8(void),option_u32(void),option_u64(void),option_ref(void),field_first(void),field_second(void),field_third(void);
int main(void){printf("%zu %zu %zu %zu %zu %zu %zu\n",option_u8(),option_u32(),option_u64(),option_ref(),field_first(),field_second(),field_third());return 0;}
