#![feature(rustc_attrs)]
#[rustc_comptime]fn id(x:u32)->u32{x}
const C:fn()->u32=||id(7);
#[unsafe(no_mangle)]pub extern "C" fn observed()->u32{C()}
