#![feature(rustc_attrs)]
#![feature(const_trait_impl)]const trait A{fn n()->u32;}struct S;fn ordinary()->u32{7}#[rustc_reservation_impl="future impl"]const impl A for S{fn n()->u32{ordinary()}}
