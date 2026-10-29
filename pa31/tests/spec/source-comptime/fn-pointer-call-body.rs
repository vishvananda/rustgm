#![feature(rustc_attrs)]
const fn next(x:u32)->u32{x+1}
#[rustc_comptime]fn id(x:u32)->u32{next(x)}
#[unsafe(no_mangle)]pub extern "C" fn observed()->u32{let f:fn(u32)->u32=id;f(6)}
