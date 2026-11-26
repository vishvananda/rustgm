#[lang="freeze"]pub unsafe auto trait Freeze{}
impl<T>Clone for *const T{fn clone(&self)->Self{*self}}impl<T>Copy for *const T{}
impl<T:Copy,const N:usize>Clone for [T;N]{fn clone(&self)->Self{*self}}impl<T:Copy,const N:usize>Copy for [T;N]{}
impl Clone for u32{fn clone(&self)->Self{*self}}impl Copy for u32{}
impl Clone for i64{fn clone(&self)->Self{*self}}impl Copy for i64{}
impl Clone for f64{fn clone(&self)->Self{*self}}impl Copy for f64{}
impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{}
#[repr(simd)]pub struct Narrow128([u32;4]);
impl Clone for Narrow128{fn clone(&self)->Self{*self}}impl Copy for Narrow128{}
#[repr(simd)]pub struct __m256i([i64;4]);
impl Clone for __m256i{fn clone(&self)->Self{*self}}impl Copy for __m256i{}
#[repr(simd)]pub struct __m256d([f64;4]);
impl Clone for __m256d{fn clone(&self)->Self{*self}}impl Copy for __m256d{}
#[repr(simd)]pub struct __m256([f32;8]);
impl Clone for __m256{fn clone(&self)->Self{*self}}impl Copy for __m256{}
#[repr(simd)]pub struct __m512i([i64;8]);
impl Clone for __m512i{fn clone(&self)->Self{*self}}impl Copy for __m512i{}
#[repr(simd)]pub struct __m512d([f64;8]);
impl Clone for __m512d{fn clone(&self)->Self{*self}}impl Copy for __m512d{}
#[repr(simd)]pub struct __m512([f32;16]);
impl Clone for __m512{fn clone(&self)->Self{*self}}impl Copy for __m512{}
#[cfg(target_pointer_width="64")]
macro_rules! vps{($inst1:expr,$inst2:expr)=>{concat!($inst1," [{p}]",$inst2)};}
#[cfg(target_pointer_width="64")]
macro_rules! vpl{($inst:expr)=>{concat!($inst,", [{p}]")};}
// ORIGINAL__mm256_stream_si256
// ORIGINAL__mm256_stream_pd
// ORIGINAL__mm256_stream_ps
#[cfg(not(older_profile))]
// ORIGINAL__mm256_stream_load_si256
#[cfg(not(older_profile))]
// ORIGINAL__mm512_stream_ps
#[cfg(not(older_profile))]
// ORIGINAL__mm512_stream_pd
#[cfg(not(older_profile))]
// ORIGINAL__mm512_stream_si512
#[cfg(not(older_profile))]
// ORIGINAL__mm512_stream_load_si512
#[cfg(not(older_profile))]
#[target_feature(enable="avx")]
pub unsafe fn pass256<T>(value:__m256i)->__m256i{let result:__m256i;unsafe{crate::arch::asm!("vmovdqa {result}, {value}",result=out(ymm_reg)result,value=in(ymm_reg)value,options(pure,nomem,nostack,preserves_flags));}result}
#[cfg(not(older_profile))]
#[target_feature(enable="avx")]
pub unsafe fn narrow128_ymm<T>(value:Narrow128)->Narrow128{let result:Narrow128;unsafe{crate::arch::asm!("vmovdqa {result}, {value}",result=out(ymm_reg)result,value=in(ymm_reg)value,options(pure,nomem,nostack,preserves_flags));}result}
#[cfg(not(older_profile))]
#[target_feature(enable="avx512f")]
pub unsafe fn pass512<T>(value:__m512i)->__m512i{let result:__m512i;unsafe{crate::arch::asm!("vmovdqa32 {result}, {value}",result=out(zmm_reg)result,value=in(zmm_reg)value,options(pure,nomem,nostack,preserves_flags));}result}
#[cfg(not(older_profile))]
#[target_feature(enable="avx512f")]
pub unsafe fn narrow128_zmm<T>(value:Narrow128)->Narrow128{let result:Narrow128;unsafe{crate::arch::asm!("vmovdqa32 {result}, {value}",result=out(zmm_reg)result,value=in(zmm_reg)value,options(pure,nomem,nostack,preserves_flags));}result}
#[cfg(not(older_profile))]
#[target_feature(enable="avx512f")]
pub unsafe fn narrow256_zmm<T>(value:__m256i)->__m256i{let result:__m256i;unsafe{crate::arch::asm!("vmovdqa32 {result}, {value}",result=out(zmm_reg)result,value=in(zmm_reg)value,options(pure,nomem,nostack,preserves_flags));}result}
#[cfg(not(older_profile))]
#[target_feature(enable="avx")]
pub unsafe fn load256<T>(p:*const __m256i)->__m256i{let result:__m256i;unsafe{crate::arch::asm!("vmovdqa {result}, [{p}]",result=out(ymm_reg)result,p=in(reg)p,options(pure,readonly,nostack,preserves_flags));}result}
#[cfg(not(older_profile))]
#[target_feature(enable="avx512f")]
pub unsafe fn load512<T>(p:*const __m512i)->__m512i{let result:__m512i;unsafe{crate::arch::asm!("vmovdqa32 {result}, [{p}]",result=out(zmm_reg)result,p=in(reg)p,options(pure,readonly,nostack,preserves_flags));}result}
pub fn ordinary<T>()->u32{99}
