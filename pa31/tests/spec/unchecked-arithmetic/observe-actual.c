#include <stdio.h>
#include <stdint.h>
extern uintptr_t actual(void);int main(void){printf("%llu\n",(unsigned long long)actual());}
