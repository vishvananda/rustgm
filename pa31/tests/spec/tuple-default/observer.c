#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
extern uintptr_t observed(void);
void initialized(uintptr_t value){printf("d%" PRIuPTR ";",value);}
void fields(uint8_t kind,uintptr_t a,uintptr_t b){printf("%c%" PRIuPTR ":%" PRIuPTR ";",kind?'r':'p',a,b);}
int main(void){printf("%" PRIuPTR "\n",observed());return 0;}
