#include <stdint.h>
#include <stdio.h>
uintptr_t read(void);uintptr_t write(void);int main(void){printf("%lu %lu\n",(unsigned long)read(),(unsigned long)write());}
