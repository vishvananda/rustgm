#include <stdint.h>
#include <stdio.h>
extern uint64_t native_observed(void);
int main(void){uint64_t value=native_observed();printf("%llu\n",(unsigned long long)value);return value==7?0:1;}
