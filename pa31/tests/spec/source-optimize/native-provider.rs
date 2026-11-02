#![feature(optimize_attribute)]
#[optimize(size)]fn a(x:u32)->u32{x+1}
#[optimize(speed)]fn b(x:u32)->u32{x*2}
#[optimize(none)]#[inline(never)]fn c(x:u32)->u32{x-3}
#[unsafe(no_mangle)]pub extern "C" fn native_observed()->u64{c(b(a(6))) as u64}
