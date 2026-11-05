#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
uint32_t pick(bool,const uint32_t*);uint32_t pick_reverse(bool,const uint32_t*);size_t tagged_size(void);
int main(void){volatile uint32_t input=23;uint32_t value=input;printf("%u %u %u %u %zu\n",pick(true,&value),pick(false,&value),pick_reverse(true,&value),pick_reverse(false,&value),tagged_size());}
