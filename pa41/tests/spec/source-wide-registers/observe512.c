#include <stdint.h>
#include <stdio.h>
#include <string.h>
void obs_store_i(uintptr_t,uintptr_t),obs_store_d(uintptr_t,uintptr_t),obs_store_f(uintptr_t,uintptr_t),obs_pass(uintptr_t,uintptr_t),obs_narrow128(uintptr_t,uintptr_t),obs_load(uintptr_t,uintptr_t);
uint32_t obs_ordinary(void);
static _Alignas(64) unsigned char input[64];
static void check(void(*call)(uintptr_t,uintptr_t),unsigned bytes){
    _Alignas(64) unsigned char buffer[192],saved[64];memset(buffer,77,sizeof buffer);memcpy(saved,input,64);
    call((uintptr_t)input,(uintptr_t)(buffer+64));__asm__ volatile("sfence" ::: "memory");
    int ok=!memcmp(input,saved,64);
    for(unsigned i=0;i<192;i++){unsigned char expected=i>=64&&i<64+bytes?input[i-64]:77;if(buffer[i]!=expected)ok=0;}
    printf("%d ",ok);
}
void obs_narrow256(uintptr_t,uintptr_t);
int main(void){for(unsigned i=0;i<64;i++)input[i]=(unsigned char)(i*73+11);check(obs_store_i,64);check(obs_store_d,64);check(obs_store_f,64);check(obs_pass,64);check(obs_narrow128,16);check(obs_narrow256,32);check(obs_load,64);input[0]=5;input[63]=229;check(obs_load,64);printf("%u\n",obs_ordinary());return 0;}
