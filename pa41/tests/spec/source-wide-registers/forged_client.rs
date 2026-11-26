#![feature(no_core,asm_native_fixture)]
#![no_core]
extern crate base;
unsafe extern "C" {fn source()->usize;fn destination()->usize;}
#[target_feature(enable="avx512f")]
unsafe fn stronger(s:usize,d:usize){unsafe{*(d as *mut base::Narrow128)=base::narrow128_ymm::<u32>(*(s as *const base::Narrow128));}}
#[unsafe(no_mangle)]pub unsafe extern "C" fn probe(s:usize,d:usize){unsafe{stronger(s,d);}}
fn main(){unsafe{probe(source(),destination());}}
