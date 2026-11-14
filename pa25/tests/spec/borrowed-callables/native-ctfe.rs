#![feature(const_trait_impl,const_destruct)]
#![allow(dead_code,unused_variables)]
pub struct Borrowed<'a,F>{pub f:&'a mut F}
pub const fn apply<F:[const]FnMut(u8)->u8>(x:&mut Borrowed<'_,F>,value:u8)->u8{(x.f)(value)}
pub const fn pass(value:u8)->u8{value}
pub const VALUE:u8={let mut f=pass;let mut x=Borrowed{f:&mut f};apply(&mut x,41)};
#[unsafe(no_mangle)]pub extern "C" fn entry()->u8{VALUE}
#[unsafe(no_mangle)]pub extern "C" fn probe(value:u8)->u8{let mut f=pass;let mut x=Borrowed{f:&mut f};apply(&mut x,value)}
fn main(){}
