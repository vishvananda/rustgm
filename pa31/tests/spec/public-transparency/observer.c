#include <inttypes.h>
#include <stdio.h>
extern uint64_t observed(void);
int main(void){printf("%" PRIu64 "\n",observed());return 0;}
