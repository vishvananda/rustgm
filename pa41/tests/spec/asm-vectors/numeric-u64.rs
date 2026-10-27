#![feature(repr_simd)]
use core::arch::asm;
#[derive(Clone,Copy)]#[repr(simd)]struct V([u64;2]);fn f(x:V){unsafe{asm!("/* {0} */",in(xmm_reg)x);}}
