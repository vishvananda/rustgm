#![feature(repr_simd)]
use core::arch::asm;
#[derive(Clone,Copy)]#[repr(simd)]struct V([u32;4]);fn f(x:V){unsafe{asm!("/* {0} */",in(xmm_reg)x);}}
