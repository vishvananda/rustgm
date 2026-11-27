#![feature(repr_simd)]
use core::arch::asm;
#[derive(Clone,Copy)]#[repr(simd)]struct V([f32;8]);#[target_feature(enable="avx512f")]fn f(x:V){unsafe{asm!("/* {0} */",in(zmm_reg)x);}}
