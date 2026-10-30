#![feature(rustc_attrs)]
trait A{fn n()->u32;}struct S;
#[rustc_reservation_impl("future")]impl A for S{fn n()->u32{99}}
