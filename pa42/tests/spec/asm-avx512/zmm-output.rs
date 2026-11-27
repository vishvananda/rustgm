#![feature(repr_simd)]
use core::arch::asm;
#[derive(Clone,Copy)]#[repr(simd)]struct V([f32;16]);#[target_feature(enable="avx512f")]fn f()->V{let x:V;unsafe{asm!("/* {0} */",out(zmm_reg)x);}x}
