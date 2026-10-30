#![feature(rustc_attrs)]
#[rustc_no_writable]fn write(x:&mut u32)->u32{*x=7;*x}
#[unsafe(no_mangle)]pub extern "C" fn observed_writable()->u32{let mut x=1;write(&mut x)}
