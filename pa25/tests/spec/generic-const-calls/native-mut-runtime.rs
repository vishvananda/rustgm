#![feature(const_trait_impl,const_destruct,const_cmp)]
use core::marker::Destruct;
pub const fn apply<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F,x:u8)->u8{f(x)}
const fn id(x:u8)->u8{x}
pub const VALUE:u8=apply(id,41u8);
#[unsafe(no_mangle)]pub extern "C" fn entry()->u8{VALUE}
fn main(){}

#[unsafe(no_mangle)]pub extern "C" fn probe(x:u8)->u8{apply(id,x)}
