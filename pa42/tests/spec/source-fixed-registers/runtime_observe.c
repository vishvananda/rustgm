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
void obs_stream(uintptr_t,int32_t);
uint64_t obs_narrow(uint64_t);
static void streams(void) {
    const int32_t inputs[4] = {0, -1, INT32_MIN, INT32_MAX};
    for (unsigned i=0;i<4;i++) {
        struct {int32_t before,value,after;} buffer = {123,456,789};
        obs_stream((uintptr_t)&buffer.value,inputs[i]);
        __asm__ volatile("sfence" ::: "memory");
        printf("%d ",buffer.before==123 && buffer.value==inputs[i] && buffer.after==789);
    }
    printf("%d\n",obs_narrow(UINT64_C(0xfedcba9887654321))==UINT64_C(0x87654321));
}
int main(void) { check(0,0);check(7,0);check(7,1);printf("%d %u %d %u %u ",obs_signed(-2),obs_ordinary(),preserved(),obs_overlap(),obs_reverse());streams();return 0; }
