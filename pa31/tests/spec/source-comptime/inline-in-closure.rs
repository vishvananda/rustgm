#![feature(rustc_attrs)]
#[rustc_comptime]fn id(x:u32)->u32{x}
#[unsafe(no_mangle)]pub extern "C" fn observed()->u32{let f=||const{id(7)};f()}
