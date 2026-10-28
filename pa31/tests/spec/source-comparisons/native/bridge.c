#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <inttypes.h>
extern bool source_eq(uintptr_t,uintptr_t);
extern bool source_ne(uintptr_t,uintptr_t);
extern bool source_lt(uintptr_t,uintptr_t);
extern bool source_le(uintptr_t,uintptr_t);
extern bool source_gt(uintptr_t,uintptr_t);
extern bool source_ge(uintptr_t,uintptr_t);
extern bool ufcs_eq(uintptr_t,uintptr_t);
extern bool ufcs_ne(uintptr_t,uintptr_t);
extern bool ufcs_lt(uintptr_t,uintptr_t);
extern bool ufcs_le(uintptr_t,uintptr_t);
extern bool ufcs_gt(uintptr_t,uintptr_t);
extern bool ufcs_ge(uintptr_t,uintptr_t);

extern int8_t partial_cmp(uintptr_t,uintptr_t),intrinsic_cmp(uintptr_t,uintptr_t),partial_signed(int64_t,int64_t),intrinsic_signed(int64_t,int64_t),partial_char(uint32_t,uint32_t),constant_signed(void),constant_unsigned(void),constant_char(void),intrinsic_order(void);
extern bool count_condition(const uintptr_t*),operator_order(void);
uintptr_t observed_left(void){fputs("L;",stdout);return 7;}
uintptr_t observed_right(void){fputs("R;",stdout);return 9;}
int main(void){uintptr_t pairs[][2]={{0,0},{0,1},{1,0},{65535,65536},{UINTPTR_MAX,0}};
for(unsigned i=0;i<5;i++){uintptr_t a=pairs[i][0],b=pairs[i][1];
printf("%d%d%d%d%d%d/%d%d%d%d%d%d:%d:%d\n",source_eq(a,b),source_ne(a,b),source_lt(a,b),source_le(a,b),source_gt(a,b),source_ge(a,b),ufcs_eq(a,b),ufcs_ne(a,b),ufcs_lt(a,b),ufcs_le(a,b),ufcs_gt(a,b),ufcs_ge(a,b),partial_cmp(a,b),intrinsic_cmp(a,b));}
int64_t signed_pairs[][2]={{INT64_MIN,INT64_MAX},{-1,0},{0,-1},{INT64_MAX,INT64_MAX}};
for(unsigned i=0;i<4;i++)printf("s:%d:%d\n",partial_signed(signed_pairs[i][0],signed_pairs[i][1]),intrinsic_signed(signed_pairs[i][0],signed_pairs[i][1]));
printf("c:%d:%d:%d\n",partial_char(0,0),partial_char(0,0x10ffff),partial_char(0x10ffff,0));
printf("k:%d:%d:%d\n",constant_signed(),constant_unsigned(),constant_char());
uintptr_t count[]={0,65535,65536,UINTPTR_MAX};for(unsigned i=0;i<4;i++)printf("t:%d\n",count_condition(&count[i]));
bool order=operator_order();printf("o:%d\n",order);int8_t tag=intrinsic_order();printf("i:%d\n",tag);return 0;}
