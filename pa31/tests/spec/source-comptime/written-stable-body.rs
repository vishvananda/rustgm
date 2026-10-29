#![feature(rustc_attrs,staged_api)]
#![unstable(feature="comptime_fixture",issue="none")]
#[unstable(feature="helper_api",issue="none")]#[rustc_const_unstable(feature="helper_const",issue="none")]pub const fn helper(x:u32)->u32{x}
#[stable(feature="always_api",since="1.0.0")]#[rustc_const_stable(feature="always_api",since="1.0.0")]
pub const fn id(x:u32)->u32{helper(x)}
const VALUE:u32=id(7);
#[unsafe(no_mangle)]pub extern "C" fn observed()->u32{VALUE}
