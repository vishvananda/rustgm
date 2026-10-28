#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
extern uintptr_t span_observed(void);
void event(uint8_t kind,uintptr_t start,uintptr_t len){printf("%u:%" PRIuPTR ":%" PRIuPTR ";",kind,start,len);}
int main(void){printf("%" PRIuPTR "\n",span_observed());return 0;}
