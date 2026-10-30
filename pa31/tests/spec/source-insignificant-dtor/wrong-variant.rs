#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
enum E{#[rustc_insignificant_dtor]A}
