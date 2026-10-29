#![feature(allow_internal_unstable)]
#[allow_internal_unstable(rustc_attrs)]macro_rules! make{()=>{#[rustc_comptime]fn id(x:u32)->u32{x}}}
make!();
const VALUE:u32=id(7);
#[unsafe(no_mangle)]pub extern "C" fn observed()->u32{VALUE}
