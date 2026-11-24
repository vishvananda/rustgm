#![feature(no_core, intrinsics)]
#![no_core]
extern crate base;
#[unsafe(no_mangle)]
pub extern "C" fn observed_size() -> usize { const { base::size_of::<base::Layout>() } }
#[unsafe(no_mangle)]
pub extern "C" fn observed_alignment() -> usize { const { base::align_of::<base::Layout>() } }
