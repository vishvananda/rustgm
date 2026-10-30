#include <stdint.h>
#include <stdio.h>
struct S{uint64_t v;};extern uint64_t read_value(struct S*);int main(void){struct S s={7};printf("%llu\n",(unsigned long long)read_value(&s));return 0;}
