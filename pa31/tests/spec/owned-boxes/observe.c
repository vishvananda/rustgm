#include <stdio.h>
#include <stdlib.h>
extern unsigned char run(void);
void *host_alloc(unsigned long size){printf("alloc %lu\n",size);return malloc(size?size:1);}
void host_free(void *pointer){printf("free\n");free(pointer);}
void host_drop(unsigned char value){printf("drop %u\n",(unsigned)value);}
int main(void){unsigned value=run();printf("value %u\n",value);return 0;}
