#include <stdint.h>
#include <stdio.h>
extern uint64_t native_observed(void);
int main(void){uint64_t v=native_observed();printf("%llu\n",(unsigned long long)v);return v==2408?0:1;}
