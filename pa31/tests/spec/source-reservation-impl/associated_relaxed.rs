#![feature(rustc_attrs)]
trait A{type X:?Sized;}struct S;#[rustc_reservation_impl="future impl"]impl A for S{type X=u8;}
