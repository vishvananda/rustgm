#![feature(repr_simd)]
use core::arch::asm;
#[target_feature(enable="avx512bw")]fn f(x:f32){unsafe{asm!("/* {0} */",in(kreg)x);}}
