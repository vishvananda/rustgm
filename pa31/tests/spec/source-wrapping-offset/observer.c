#include <inttypes.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
const uint32_t *offset_word(const uint32_t *,intptr_t);
const unsigned char *offset_byte(const unsigned char *,intptr_t);
const void *offset_zero(const void *,intptr_t);
uint32_t read_back(const uint32_t *,intptr_t,intptr_t);
uint32_t const_round(void);uint32_t const_max_back(void);uint32_t const_min_stay(void);
uint8_t const_zero(void);const unsigned char *const_raw(void);
int main(void){
uint32_t values[4]={0x11223344,0x22334455,0x33445566,0x44556677};unsigned char zero=0;
uintptr_t base=(uintptr_t)values;
const uint32_t *outside=offset_word(values,100);
printf("pos=%" PRIuPTR ";neg=%" PRIuPTR ";back=%" PRIuPTR ";",(uintptr_t)offset_word(values,2)-base,(uintptr_t)offset_word(values+2,-2)-base,(uintptr_t)offset_word(outside,-100)-base);
printf("wmax=%016" PRIxPTR ";wmin=%016" PRIxPTR ";",(uintptr_t)offset_word(values,INTPTR_MAX)-base,(uintptr_t)offset_word(values,INTPTR_MIN)-base);
printf("bmax=%016" PRIxPTR ";bmin=%016" PRIxPTR ";",(uintptr_t)offset_byte((const unsigned char *)values,INTPTR_MAX)-base,(uintptr_t)offset_byte((const unsigned char *)values,INTPTR_MIN)-base);
printf("zmax=%" PRIuPTR ";zmin=%" PRIuPTR ";",(uintptr_t)offset_zero(&zero,INTPTR_MAX)-(uintptr_t)&zero,(uintptr_t)offset_zero(&zero,INTPTR_MIN)-(uintptr_t)&zero);
printf("null=%" PRIuPTR ":%" PRIuPTR ";raw=%016" PRIxPTR ";",(uintptr_t)offset_byte(NULL,0),(uintptr_t)offset_byte(NULL,3),(uintptr_t)offset_byte((const unsigned char *)(uintptr_t)0x1000,-1));
printf("read=%08" PRIx32 ";const=%08" PRIx32 ":%08" PRIx32 ":%08" PRIx32 ":%" PRIu8 ";constraw=%" PRIuPTR "\n",read_back(values,100,-100),const_round(),const_max_back(),const_min_stay(),const_zero(),(uintptr_t)const_raw());
return 0;}
