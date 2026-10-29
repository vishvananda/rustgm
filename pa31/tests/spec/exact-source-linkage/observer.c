#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
extern uint8_t byte(void); extern __uint128_t wide(void); extern __int128 signed_value(void) __asm__("signed");
extern bool flag(void); extern uint32_t letter(void); extern float single(void); extern double double_value(void) __asm__("double");
int main(void) { float f=single(); double d=double_value(); uint32_t fb; uint64_t db; memcpy(&fb,&f,sizeof fb); memcpy(&db,&d,sizeof db);
__uint128_t w=wide(),s=(__uint128_t)signed_value();
printf("%u;%016lx%016lx;%016lx%016lx;%u;%u;%08x;%016lx\n",(unsigned)byte(),(unsigned long)(w>>64),(unsigned long)w,(unsigned long)(s>>64),(unsigned long)s,(unsigned)flag(),(unsigned)letter(),fb,(unsigned long)db); return 0; }
