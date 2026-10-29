#![feature(rustc_attrs)]
#[rustc_pub_transparent]#[repr(transparent)]pub struct Value{inner:u64}
#[unsafe(no_mangle)]pub extern "C" fn observed()->u64{Value{inner:7}.inner}
