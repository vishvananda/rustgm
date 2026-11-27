#![feature(repr_simd)]
#![feature(f16)]
use core::arch::asm;
#[derive(Clone,Copy)]#[repr(simd)]struct V([f16;16]);#[target_feature(enable="avx")]fn f(x:V){unsafe{asm!("/* {0} */",in(ymm_reg)x);}}
