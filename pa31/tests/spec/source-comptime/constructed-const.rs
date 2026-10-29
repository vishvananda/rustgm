#![feature(rustc_attrs)]
#[rustc_comptime]fn id(x:u32)->u32{x}
struct Pair(u32);
#[rustc_comptime]fn build()->Pair{Pair(id(7))}
const VALUE:Pair=build();
#[unsafe(no_mangle)]pub extern "C" fn observed()->u32{VALUE.0}
