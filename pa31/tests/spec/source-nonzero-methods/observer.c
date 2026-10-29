#include <stdint.h>
#include <stdio.h>
extern uint8_t nz_new(uint8_t),nz_get(uint8_t),const_none(void),const_one(void),const_max(void),const_get(void);
int main(void){printf("%u %u %u %u %u %u %u\n",(unsigned)nz_get(nz_new(0)),(unsigned)nz_get(nz_new(1)),(unsigned)nz_get(nz_new(255)),(unsigned)const_none(),(unsigned)const_one(),(unsigned)const_max(),(unsigned)const_get());return 0;}
