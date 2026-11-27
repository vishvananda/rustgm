#![feature(repr_simd)]
use core::arch::asm;
#[target_feature(enable="avx512fp16")]fn f(x:u8){unsafe{asm!("/* {0} */",in(kreg)x);}}
