#include <stdint.h>
#include <stdio.h>
extern unsigned char run(void);
extern uintptr_t wrapper_size(void),wrapper_align(void);
int main(void){printf("%u %llu %llu\n",(unsigned)run(),(unsigned long long)wrapper_size(),(unsigned long long)wrapper_align());}
