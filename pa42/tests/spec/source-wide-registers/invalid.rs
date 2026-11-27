#![feature(no_core,asm_native_fixture)]
#![no_core]
extern crate base;
pub unsafe fn wrong(){unsafe{base::pass256::<u32>(42_u32);}}
