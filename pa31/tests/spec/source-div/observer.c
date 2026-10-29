#include <stdint.h>
#include <stdio.h>
extern uint8_t op_u8(uint8_t,uint8_t),ufcs_u8(uint8_t,uint8_t),constant_u8(void);
extern uint16_t op_u16(uint16_t,uint16_t),ufcs_u16(uint16_t,uint16_t),constant_u16(void);
extern uint32_t op_u32(uint32_t,uint32_t),ufcs_u32(uint32_t,uint32_t),constant_u32(void);
extern uint64_t op_u64(uint64_t,uint64_t),ufcs_u64(uint64_t,uint64_t),constant_u64(void);
extern unsigned __int128 op_u128(unsigned __int128,unsigned __int128),ufcs_u128(unsigned __int128,unsigned __int128),constant_u128(void);
extern uintptr_t op_usize(uintptr_t,uintptr_t),ufcs_usize(uintptr_t,uintptr_t),constant_usize(void);
extern int8_t op_i8(int8_t,int8_t),ufcs_i8(int8_t,int8_t),constant_i8(void);
extern int16_t op_i16(int16_t,int16_t),ufcs_i16(int16_t,int16_t),constant_i16(void);
extern int32_t op_i32(int32_t,int32_t),ufcs_i32(int32_t,int32_t),constant_i32(void);
extern int64_t op_i64(int64_t,int64_t),ufcs_i64(int64_t,int64_t),constant_i64(void);
extern __int128 op_i128(__int128,__int128),ufcs_i128(__int128,__int128),constant_i128(void);
extern intptr_t op_isize(intptr_t,intptr_t),ufcs_isize(intptr_t,intptr_t),constant_isize(void);
extern float op_f32(float,float),ufcs_f32(float,float),constant_f32(void);
extern double op_f64(double,double),ufcs_f64(double,double),constant_f64(void);
int main(void){
 if(op_u8(9,2)!=4||ufcs_u8(9,2)!=4||constant_u8()!=4)return 1;
 if(op_u16(9,2)!=4||ufcs_u16(9,2)!=4||constant_u16()!=4)return 1;
 if(op_u32(9,2)!=4||ufcs_u32(9,2)!=4||constant_u32()!=4)return 1;
 if(op_u64(9,2)!=4||ufcs_u64(9,2)!=4||constant_u64()!=4)return 1;
 if(op_u128(9,2)!=4||ufcs_u128(9,2)!=4||constant_u128()!=4)return 1;
 if(op_usize(9,2)!=4||ufcs_usize(9,2)!=4||constant_usize()!=4)return 1;
 if(op_i8(9,2)!=4||ufcs_i8(9,2)!=4||constant_i8()!=4)return 1;
 if(op_i8(-17,3)!=-5||ufcs_i8(-17,3)!=-5)return 2;
 if(op_i16(9,2)!=4||ufcs_i16(9,2)!=4||constant_i16()!=4)return 1;
 if(op_i16(-17,3)!=-5||ufcs_i16(-17,3)!=-5)return 2;
 if(op_i32(9,2)!=4||ufcs_i32(9,2)!=4||constant_i32()!=4)return 1;
 if(op_i32(-17,3)!=-5||ufcs_i32(-17,3)!=-5)return 2;
 if(op_i64(9,2)!=4||ufcs_i64(9,2)!=4||constant_i64()!=4)return 1;
 if(op_i64(-17,3)!=-5||ufcs_i64(-17,3)!=-5)return 2;
 if(op_i128(9,2)!=4||ufcs_i128(9,2)!=4||constant_i128()!=4)return 1;
 if(op_i128(-17,3)!=-5||ufcs_i128(-17,3)!=-5)return 2;
 if(op_isize(9,2)!=4||ufcs_isize(9,2)!=4||constant_isize()!=4)return 1;
 if(op_isize(-17,3)!=-5||ufcs_isize(-17,3)!=-5)return 2;
 if(op_f32(9,2)!=4.5||ufcs_f32(9,2)!=4.5||constant_f32()!=4.5)return 1;
 if(op_f64(9,2)!=4.5||ufcs_f64(9,2)!=4.5||constant_f64()!=4.5)return 1;
 puts("source division 14 scalar families");return 0;}
