#include <stdint.h>
#include <stdio.h>
struct S{uint64_t v;};uint64_t change(struct S* p){p->v=9;return p->v;}extern uint64_t native_observed(void);int main(void){printf("%llu\n",(unsigned long long)native_observed());return 0;}
