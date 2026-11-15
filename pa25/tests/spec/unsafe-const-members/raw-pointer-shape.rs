#![feature(const_trait_impl)]
#![allow(dead_code,unused_variables,unused_unsafe)]
pub const trait Base<U>{fn read(value:U)->u8;}
pub const trait RawBridge<B>{unsafe fn compare(lhs:*const Self,rhs:*const B,len:usize)->bool;}
const impl<A:[const]Base<B>,B>RawBridge<B>for A{unsafe fn compare(lhs:*const Self,rhs:*const B,len:usize)->bool{true}}
pub const fn apply<A:[const]Base<B>,B>(lhs:*const A,rhs:*const B,len:usize)->bool{unsafe{RawBridge::compare(lhs,rhs,len)}}
fn main(){}
