#![feature(rustc_attrs,staged_api)]
#![unstable(feature="comptime_fixture",issue="none")]
#[unstable(feature="comptime_fixture",issue="none")]
#[rustc_const_unstable(feature="comptime_const",issue="none")]
#[rustc_comptime]pub fn id(x:u32)->u32{x}
const VALUE:u32=id(7);
#[unsafe(no_mangle)]pub extern "C" fn observed()->u32{VALUE}
