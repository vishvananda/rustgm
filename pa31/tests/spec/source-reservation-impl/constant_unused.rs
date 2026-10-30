#![feature(rustc_attrs)]trait A{const V:u32;}struct S;#[rustc_reservation_impl="future impl"]impl A for S{const V:u32=7;}
