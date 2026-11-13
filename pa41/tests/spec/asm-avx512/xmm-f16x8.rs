#![feature(repr_simd)]
#![feature(f16)]
use core::arch::asm;
#[derive(Clone,Copy)]#[repr(simd)]struct V([f16;8]);#[target_feature(enable="sse")]fn f(x:V){unsafe{asm!("/* {0} */",in(xmm_reg)x);}}
