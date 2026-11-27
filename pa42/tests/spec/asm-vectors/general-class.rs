#![feature(repr_simd)]
use core::arch::asm;
#[derive(Clone,Copy)]#[repr(simd)]struct V([f32;4]);fn f(x:V){unsafe{asm!("/* {0} */",in(reg)x);}}
