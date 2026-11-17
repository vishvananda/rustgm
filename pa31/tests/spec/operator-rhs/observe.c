#include <stdio.h>
extern unsigned char run(void);
void host_deref(void){puts("deref");}
void host_mut(void){puts("mut");}
void host_add(void){puts("add");}
int main(void){printf("value %u\n",(unsigned)run());return 0;}
