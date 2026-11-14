#include <stdio.h>
#include <stdint.h>
typedef __uint128_t U;typedef __int128_t I;
extern uint8_t left_u8(uint8_t,uint32_t);extern int8_t right_i8(int8_t,U);
extern U left_u128(U,int8_t);extern I right_i128(I,int8_t);
extern intptr_t right_isize(intptr_t,uint64_t);extern uintptr_t left_usize(uintptr_t,uint8_t);
int main(void){U l=left_u128(1,127),r=(U)right_i128(-2,127);
printf("%02x %02x %016llx%016llx %016llx%016llx %016llx %016llx\n",left_u8(129,7),(uint8_t)right_i8(-2,7),(unsigned long long)(l>>64),(unsigned long long)l,(unsigned long long)(r>>64),(unsigned long long)r,(unsigned long long)right_isize(-2,63),(unsigned long long)left_usize(1,63));}
