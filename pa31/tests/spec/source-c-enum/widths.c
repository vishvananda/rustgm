#include <stdio.h>
extern unsigned char echo_u8(unsigned char);
extern signed char echo_i8(signed char);
extern unsigned short echo_u16(unsigned short);
extern short echo_i16(short);
extern unsigned int echo_u32(unsigned int);
extern int echo_i32(int);
extern unsigned long long echo_u64(unsigned long long);
extern long long echo_i64(long long);
extern unsigned __int128 echo_u128(unsigned __int128);
extern __int128 echo_i128(__int128);
extern unsigned long echo_usize(unsigned long);
extern long echo_isize(long);
int main(void){printf("%llu",(unsigned long long)echo_u8((unsigned char)(1ULL)));printf(" ");printf("%lld",(long long)echo_i8((signed char)(-3)));printf(" ");printf("%llu",(unsigned long long)echo_u16((unsigned short)(65535ULL)));printf(" ");printf("%lld",(long long)echo_i16((short)(-3)));printf(" ");printf("%llu",(unsigned long long)echo_u32((unsigned int)(4294967295ULL)));printf(" ");printf("%lld",(long long)echo_i32((int)(-3)));printf(" ");printf("%llu",(unsigned long long)echo_u64((unsigned long long)(18446744073709551615ULL)));printf(" ");printf("%lld",(long long)echo_i64((long long)(-3)));printf(" ");printf("%d",echo_u128(((unsigned __int128)1)<<80)==(((unsigned __int128)1)<<80));printf(" ");printf("%d",echo_i128(-((__int128)3))==-((__int128)3));printf(" ");printf("%llu",(unsigned long long)echo_usize((unsigned long)(17ULL)));printf(" ");printf("%lld",(long long)echo_isize((long)(-3)));puts("");return 0;}
