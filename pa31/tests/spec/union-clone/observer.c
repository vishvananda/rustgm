#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
extern uint32_t observed(void);
void bits(uint32_t first,uint32_t second,uint32_t third){printf("%08" PRIx32 ":%08" PRIx32 ":%08" PRIx32 ";",first,second,third);}
void uninitialized_copy(void){printf("u;");}
void array_bits(uint32_t first,uint32_t second){printf("a:%08" PRIx32 ":%08" PRIx32 ";",first,second);}
int main(void){printf("%" PRIu32 "\n",observed());return 0;}
