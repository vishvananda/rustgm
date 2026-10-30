#![feature(rustc_attrs)]
#![feature(negative_impls,auto_traits)]auto trait A{}struct S;#[rustc_reservation_impl="future impl"]impl !A for S{}
