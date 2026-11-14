#![feature(const_trait_impl,unboxed_closures,fn_traits)]
pub const fn invoke<F:[const]FnMut(u8)->u8>(f:&F,value:u8)->u8{f.call_mut((value,))}
fn main(){}
