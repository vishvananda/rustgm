#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>
extern uint8_t unchecked_u8(uint8_t,uint8_t);extern uint8_t wrapping_u8(uint8_t,uint8_t);extern bool overflow_u8(uint8_t,uint8_t);extern uint8_t folded_u8(void);extern bool folded_flag_u8(void);extern uint32_t cold_observed(void);
int main(void){if(unchecked_u8(9,4)!=9||wrapping_u8(0,1)!=1||overflow_u8(0,1)||folded_u8()!=9||folded_flag_u8()||cold_observed()!=7)return 1;puts("ordinary 9 1 0 const9 const0 cold7");return 0;}
