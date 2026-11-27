#![feature(repr_simd)]
use core::arch::asm;
#[derive(Clone,Copy)]#[repr(simd)]struct V([f32;8]);#[target_feature(enable="avx")]fn f(x:V){unsafe{asm!("/* {0} */",in(xmm_reg)x);}}
