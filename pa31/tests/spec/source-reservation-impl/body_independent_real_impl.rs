#![feature(rustc_attrs)]
trait A{fn n()->u32;}struct S;impl A for S{fn n()->u32{7}}#[rustc_reservation_impl="future impl"]impl A for S{fn n()->u32{<S as A>::n()}}
