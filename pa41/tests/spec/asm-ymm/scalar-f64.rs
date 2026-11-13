#![feature(repr_simd)]
use core::arch::asm;
#[target_feature(enable="avx")]fn f(x:f64){unsafe{asm!("/* {0} */",in(ymm_reg)x);}}
