#![feature(rustc_attrs,const_trait_impl)]
const trait A{type X;}struct S;impl A for S{type X=u32;}#[rustc_reservation_impl="future impl"]const impl A for S{type X=u8;}
