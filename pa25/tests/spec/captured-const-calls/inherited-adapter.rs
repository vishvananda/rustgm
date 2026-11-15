#![feature(const_closures,const_trait_impl,const_destruct)]
#![allow(dead_code,unused_variables)]
use core::marker::Destruct;
pub struct Packet<T>(pub T);
impl<T>Packet<T>{pub const fn wrap<A,B,F>(mut f:F)->impl [const]FnMut(A,B)->Self+[const]Destruct
where F:[const]FnMut(A,B)->T+[const]Destruct{const move |a,b| Packet(f(a,b))}}
fn main(){}
