#![feature(rustc_attrs)]
trait A{type X;}struct S;impl A for S{type X=u32;}#[rustc_reservation_impl="future impl"]impl A for S{type X=u8;}pub fn call(_: <S as A>::X){}
