#![feature(repr_simd)]
use core::arch::asm;
#[target_feature(enable="avx512f")]fn f(x:u16){unsafe{asm!("/* {0:x} */",in(kreg)x);}}
