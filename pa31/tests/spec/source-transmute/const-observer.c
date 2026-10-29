#include <stddef.h>
#include <stdio.h>
extern size_t observed(void);
int main(void){printf("%zu\n",observed());return 0;}
