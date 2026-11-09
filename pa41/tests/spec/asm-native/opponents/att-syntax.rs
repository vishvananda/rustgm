#![feature(repr_simd)]
use core::arch::asm;
#[derive(Clone,Copy)]#[repr(simd)]pub struct V([f32;4]);pub fn f(p:*mut f32,a:V){unsafe{asm!("movntps {a},({p})",p=in(reg)p,a=in(xmm_reg)a,options(att_syntax,nostack,preserves_flags));}}
