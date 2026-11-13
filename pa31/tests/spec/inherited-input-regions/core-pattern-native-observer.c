#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>
bool equal_u8(const uint8_t *haystack,const uint8_t *needle){return *haystack==*needle;}
extern uint8_t search_needle(const uint8_t **,const uint8_t *);
extern uint8_t search_haystack(const uint8_t **,const uint8_t *);
extern bool contains(const uint8_t **,const uint8_t *);
int main(void){uint8_t n=13,h=29;const uint8_t *p=&n;printf("%u %u %u ",search_needle(&p,&h),search_haystack(&p,&h),contains(&p,&h));h=13;printf("%u %u %u\n",search_needle(&p,&h),search_haystack(&p,&h),contains(&p,&h));}
