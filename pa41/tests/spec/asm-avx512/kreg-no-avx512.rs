#![feature(repr_simd)]
use core::arch::asm;
#[target_feature(enable="avx2")]fn f(x:u8){unsafe{asm!("/* {0} */",in(kreg)x);}}
