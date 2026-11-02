#include <stdint.h>
#include <stdio.h>
extern void iteration_drop(void);
extern void iteration_break(void);
void mark(uint8_t value){printf("%u ",value);}
int main(void){iteration_drop();puts("");iteration_break();puts("");return 0;}
