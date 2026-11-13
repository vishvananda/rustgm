#![feature(repr_simd)]
use core::arch::asm;
#[derive(Clone,Copy)]#[repr(simd)]struct V([i32;4]);#[target_feature(enable="avx")]fn f(x:V){unsafe{asm!("/* {0} */",in(ymm_reg)x);}}
