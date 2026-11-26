#![feature(no_core,asm_native_fixture)]
#![no_core]
extern crate base;
pub fn wrong()->u32 {base::query::<u32>(true,0).eax}
