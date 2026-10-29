#![feature(rustc_attrs)]
#[rustc_comptime]fn id(x:u32)->u32{x}
fn run<F:Fn(u32)->u32>(f:F)->u32{f(7)}
#[unsafe(no_mangle)]pub extern "C" fn observed()->u32{run(id)}
