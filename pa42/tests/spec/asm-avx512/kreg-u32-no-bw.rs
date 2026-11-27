#![feature(repr_simd)]
use core::arch::asm;
#[target_feature(enable="avx512f")]fn f(x:u32){unsafe{asm!("/* {0} */",in(kreg)x);}}
