#include <stdint.h>
#include <stdio.h>
extern uintptr_t native_size(void);
extern uintptr_t native_align(void);
int main(void){uintptr_t s=native_size(),a=native_align();printf("%lu %lu\n",(unsigned long)s,(unsigned long)a);return s==24&&a==8?0:1;}
