#![feature(const_trait_impl)]
#![allow(dead_code,unused_variables,unused_unsafe)]
pub const trait Base<U>{fn read(value:U)->u8;}
pub const trait Bridge<U>{unsafe fn read(value:U)->u8;}
const impl<T:[const]Base<U>,U>Bridge<U>for T{unsafe fn read(value:U)->u8{<T as Base<U>>::read(value)}}
pub const fn apply<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub struct Tag;
const impl Base<u8>for Tag{fn read(value:u8)->u8{value}}
pub const VALUE:u8=apply::<Tag,u8>(41);
#[unsafe(no_mangle)]pub extern "C" fn entry()->u8{VALUE}
#[unsafe(no_mangle)]pub extern "C" fn probe(value:u8)->u8{apply::<Tag,u8>(value)}
fn main(){}
