#![feature(repr_simd)]
use core::arch::asm;
#[derive(Clone,Copy)]#[repr(simd)]struct V([f32;16]);#[target_feature(enable="avx512f")]fn f(x:V){unsafe{asm!("/* {0:e} */",in(zmm_reg)x);}}
