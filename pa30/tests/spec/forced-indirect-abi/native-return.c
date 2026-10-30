#include <stdint.h>
#include <stdio.h>
struct S{uint64_t v;};extern struct S echo(struct S*);int main(void){struct S s={7};struct S r=echo(&s);printf("%llu\n",(unsigned long long)r.v);return 0;}
