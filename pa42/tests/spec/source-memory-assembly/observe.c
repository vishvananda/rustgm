#include <stdint.h>
#include <stdio.h>
#include <string.h>
void obs_load(uintptr_t,uintptr_t),obs_duplicate(uintptr_t,uintptr_t),obs_inplace(uintptr_t,uintptr_t),obs_alias(uintptr_t,uintptr_t);
uint32_t obs_read(uintptr_t),obs_ordinary(void);
struct guarded{_Alignas(16) uint32_t before[4],value[4],after[4];};
static int sentinels(const struct guarded* b){return b->before[0]==123 && b->before[3]==234 && b->after[0]==345 && b->after[3]==456;}
int main(void){
    _Alignas(16) uint32_t input[4]={0,0xffffffffu,0x80000000u,0x76543210u};
    struct guarded b={{123,0,0,234},{0,0,0,0},{345,0,0,456}};
    const uintptr_t source=(uintptr_t)input,dest=(uintptr_t)b.value;
    obs_load(source,dest);printf("%d ",!memcmp(b.value,input,16)&&sentinels(&b));
    input[0]=0x12345678u;input[3]=0xfedcba98u;
    obs_load(source,dest);printf("%d ",!memcmp(b.value,input,16)&&sentinels(&b));
    uint32_t saved[4];memcpy(saved,input,16);
    obs_duplicate(source,dest);printf("%d ",!memcmp(b.value,input,16)&&!memcmp(input,saved,16)&&sentinels(&b));
    const uint32_t zero[4]={0,0,0,0},ones[4]={0xffffffffu,0xffffffffu,0xffffffffu,0xffffffffu};
    obs_inplace(source,dest);printf("%d ",!memcmp(b.value,zero,16)&&!memcmp(input,saved,16)&&sentinels(&b));
    obs_alias(source,dest);printf("%d ",!memcmp(b.value,ones,16)&&!memcmp(input,saved,16)&&sentinels(&b));
    uint32_t scalar=123;uint32_t first=obs_read((uintptr_t)&scalar);scalar=456;
    printf("%d %u\n",first==123&&obs_read((uintptr_t)&scalar)==456,obs_ordinary());
    return 0;
}
