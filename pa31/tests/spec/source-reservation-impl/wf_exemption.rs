#![feature(rustc_attrs)]
trait Base{}trait A:Base{}struct S;#[rustc_reservation_impl="future impl"]impl A for S{}
