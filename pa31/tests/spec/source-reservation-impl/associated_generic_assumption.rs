#![feature(rustc_attrs)]
trait A{type X;}#[rustc_reservation_impl="future impl"]impl<T:A>A for T{type X=u8;}
