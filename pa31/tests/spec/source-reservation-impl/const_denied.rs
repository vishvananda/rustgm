#![feature(rustc_attrs)]
#![feature(const_trait_impl)]const trait A{fn n()->u32;}struct S;#[rustc_reservation_impl="future impl"]const impl A for S{fn n()->u32{99}}const V:u32=<S as A>::n();
