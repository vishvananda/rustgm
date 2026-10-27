#![feature(repr_simd)]
use core::arch::asm;
#[derive(Clone,Copy)]#[repr(simd)]struct V([f32;4]);fn f(x:V){let mut y:usize=0;unsafe{asm!("/* {0} */",inout(xmm_reg)x=>y);}}
