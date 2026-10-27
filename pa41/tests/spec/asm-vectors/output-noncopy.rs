#![feature(repr_simd)]
use core::arch::asm;
#[repr(simd)]struct V([f32;4]);fn f(){let x:V;unsafe{asm!("/* {0} */",out(xmm_reg)x);}}
