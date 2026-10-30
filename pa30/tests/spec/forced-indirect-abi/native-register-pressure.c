#include <stdint.h>
#include <stdio.h>
struct S{uint64_t v;};extern uint64_t pressure(uint64_t,uint64_t,uint64_t,uint64_t,uint64_t,uint64_t,struct S*,uint64_t);int main(void){struct S s={7};printf("%llu\n",(unsigned long long)pressure(1,2,3,4,5,6,&s,9));return 0;}
