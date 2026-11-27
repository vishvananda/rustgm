#![feature(repr_simd)]
use core::arch::asm;
#[repr(simd)]struct V([f32;4]);fn f(mut x:V){unsafe{asm!("/* {0} */",inout(xmm_reg)x);}}
