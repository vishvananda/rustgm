#![feature(rustc_attrs)]
trait A<X>{}trait B{}struct S;#[rustc_reservation_impl="future impl"]impl A<u16>for S{}impl<T:A<u8>>B for T{}impl B for S{}
