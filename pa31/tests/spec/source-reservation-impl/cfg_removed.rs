#![feature(rustc_attrs)]trait A{}struct S;#[cfg(any())]#[rustc_reservation_impl]#[rustc_reservation_impl=7]impl A for S{}
