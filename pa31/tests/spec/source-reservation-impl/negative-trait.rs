#![feature(rustc_attrs,negative_impls)]
trait A{}struct S;#[rustc_reservation_impl="future impl"]impl !A for S{}
