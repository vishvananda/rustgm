#![feature(repr_simd)]
use core::arch::asm;
#[target_feature(enable="avx512f")]fn f(x:f64){unsafe{asm!("/* {0} */",in(zmm_reg)x);}}
