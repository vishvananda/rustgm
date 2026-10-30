#![feature(rustc_attrs)]
trait A{fn n()->u32;}struct S;
#[rustc_reservation_impl="future impl"]impl A for S{fn n()->u32{99}}pub fn call()->u32{<S as A>::n()}
