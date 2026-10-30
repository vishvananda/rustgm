#![feature(rustc_attrs)]trait A{}struct S;#[unsafe(rustc_reservation_impl="future impl")]impl A for S{}
