#![feature(rustc_attrs)]
trait A{fn n()->u32;}#[rustc_reservation_impl="future impl"]impl<T:A>A for T{fn n()->u32{<T as A>::n()}}
