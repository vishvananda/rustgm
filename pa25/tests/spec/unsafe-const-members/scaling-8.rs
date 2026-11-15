#![feature(const_trait_impl)]
#![allow(dead_code,unused_variables,unused_unsafe)]
pub const trait Base<U>{fn read(value:U)->u8;}
pub const trait Bridge<U>{unsafe fn read(value:U)->u8;}
const impl<T:[const]Base<U>,U>Bridge<U>for T{unsafe fn read(value:U)->u8{<T as Base<U>>::read(value)}}
pub const fn apply0<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply1<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply2<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply3<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply4<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply5<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply6<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
pub const fn apply7<T:[const]Base<U>,U>(value:U)->u8{unsafe{<T as Bridge<U>>::read(value)}}
fn main(){}
