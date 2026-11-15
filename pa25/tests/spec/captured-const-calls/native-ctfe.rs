#![feature(const_closures,const_trait_impl,const_destruct)]
#![allow(dead_code,unused_variables)]
use core::marker::Destruct;
pub const fn wrap<F:[const]FnMut(u8)->u8+[const]Destruct>(mut f:F)->impl [const]FnMut(u8)->u8+[const]Destruct{const move |a| f(a)}
pub const fn pass(value:u8)->u8{value}
pub const VALUE:u8={let mut f=wrap(pass);f(41)};
#[unsafe(no_mangle)]pub extern "C" fn entry()->u8{VALUE}
#[unsafe(no_mangle)]pub extern "C" fn probe(value:u8)->u8{let mut f=wrap(pass);f(value)}
fn main(){}
