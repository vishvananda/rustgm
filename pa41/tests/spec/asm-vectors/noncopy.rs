#![feature(repr_simd)]
use core::arch::asm;
#[repr(simd)]struct V([f32;4]);fn f(x:V){unsafe{asm!("/* {0} */",in(xmm_reg)x);}}
