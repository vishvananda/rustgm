#include <stddef.h>
#include <stdio.h>
extern size_t size32(void),size64(void),actual(void),pair32(void),pair64(void);
int main(void){printf("%zu %zu %zu %zu %zu\n",size32(),size64(),actual(),pair32(),pair64());return 0;}
