#[lang="freeze"] pub unsafe auto trait Freeze {}
impl<T>Clone for *const T{fn clone(&self)->Self{*self}}impl<T>Copy for *const T{}
#[repr(simd)] pub struct __m128i([u32;4]);
impl Clone for u32{fn clone(&self)->Self{*self}}impl Copy for u32{}
impl<T:Copy,const N:usize>Clone for [T;N]{fn clone(&self)->Self{*self}}impl<T:Copy,const N:usize>Copy for [T;N]{}
impl Clone for __m128i{fn clone(&self)->Self{*self}}impl Copy for __m128i{}
#[cfg(target_pointer_width = "64")]
macro_rules! vpl {
    ($inst:expr) => { concat!($inst, ", [{p}]") };
}
// ORIGINAL_LOAD
pub unsafe fn load<T>(p:*const __m128i)->__m128i {
    let result:__m128i;
    unsafe{crate::arch::asm!("movdqa {result}, [{p}]",result=out(xmm_reg)result,p=in(reg)p,options(pure,readonly,nostack,preserves_flags));}
    result
}
pub unsafe fn duplicate<T>(value:__m128i)->__m128i {
    let result:__m128i;
    unsafe{crate::arch::asm!("movdqa {result}, {value}",result=out(xmm_reg)result,value=in(xmm_reg)value,options(pure,nomem,nostack,preserves_flags));}
    result
}
pub unsafe fn inplace<T>(mut value:__m128i)->__m128i {
    unsafe{crate::arch::asm!("pxor {value}, {value}",value=inout(xmm_reg)value,options(pure,nomem,nostack,preserves_flags));}
    value
}
pub unsafe fn alias<T>(mut value:__m128i)->__m128i {
    unsafe{crate::arch::asm!("pxor {zero}, {zero}","pcmpeqd {ones}, {ones}",zero=out(xmm_reg)value,ones=lateout(xmm_reg)value,options(pure,nomem,nostack,preserves_flags));}
    value
}
pub unsafe fn read<T>(p:*const u32)->u32 {
    let result:u32;
    unsafe{crate::arch::asm!("mov {result:e}, [{p}]",result=out(reg)result,p=in(reg)p,options(pure,readonly,nostack,preserves_flags));}
    result
}
pub fn ordinary<T>()->u32{99}
