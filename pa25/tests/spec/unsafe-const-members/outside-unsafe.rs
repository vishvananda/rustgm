#![feature(const_trait_impl)]
#![allow(dead_code,unused_variables,unused_unsafe)]
pub const trait Base<U>{fn read(value:U)->u8;}
pub const trait Bridge<U>{unsafe fn read(value:U)->u8;}
const impl<T:[const]Base<U>,U>Bridge<U>for T{unsafe fn read(value:U)->u8{<T as Base<U>>::read(value)}}
pub const fn apply<T:[const]Base<U>,U>(value:U)->u8{<T as Bridge<U>>::read(value)}
fn main(){}
