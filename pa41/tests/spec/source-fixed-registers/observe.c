#include <stdint.h>
#include <stdio.h>
#include <cpuid.h>
uint32_t obs_eax(uint32_t,uint32_t),obs_ebx(uint32_t,uint32_t),obs_ecx(uint32_t,uint32_t),obs_edx(uint32_t,uint32_t),obs_ordinary(void);
int32_t obs_signed(int32_t);
uint32_t obs_overlap(void),obs_reverse(void);
static void check(uint32_t leaf,uint32_t sub_leaf) {
    uint32_t a,b,c,d;
    __cpuid_count(leaf,sub_leaf,a,b,c,d);
    printf("%u %u %u %u ",obs_eax(leaf,sub_leaf)==a,obs_ebx(leaf,sub_leaf)==b,obs_ecx(leaf,sub_leaf)==c,obs_edx(leaf,sub_leaf)==d);
}
static int preserved(void) {
    register uint64_t saved __asm__("rbx") = UINT64_C(0x123456789abcdef0);
    __asm__ volatile("" : "+r"(saved));
    (void)obs_ebx(0,0);
    __asm__ volatile("" : "+r"(saved));
    return saved == UINT64_C(0x123456789abcdef0);
}
int main(void) { check(0,0);check(7,0);check(7,1);printf("%d %u %d %u %u\n",obs_signed(-2),obs_ordinary(),preserved(),obs_overlap(),obs_reverse());return 0; }
