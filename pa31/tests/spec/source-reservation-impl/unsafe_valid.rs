#![feature(rustc_attrs)]
unsafe trait A{}struct S;#[rustc_reservation_impl="future impl"]unsafe impl A for S{}
