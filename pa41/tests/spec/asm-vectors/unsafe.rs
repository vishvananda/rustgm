#![feature(repr_simd)]
use core::arch::asm;
#[derive(Clone,Copy)]#[repr(simd)]struct V([f32;4]);fn f(x:V){asm!("/* {0} */",in(xmm_reg)x);}
