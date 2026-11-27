#![feature(no_core,asm_native_fixture)]
#![no_core]
extern crate base;
pub unsafe fn wrong()->u32{unsafe{base::read::<u32>(42_u32)}}
