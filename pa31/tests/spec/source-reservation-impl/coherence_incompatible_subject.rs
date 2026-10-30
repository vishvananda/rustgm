#![feature(rustc_attrs)]
trait A{}trait B{}#[rustc_reservation_impl="future impl"]impl A for u16{}impl<T:A>B for T{}impl B for u8{}
