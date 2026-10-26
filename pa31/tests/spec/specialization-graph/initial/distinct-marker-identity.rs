#![allow(dead_code,unused_imports,incomplete_features,internal_features,unused_macros)]
#![feature(min_specialization,rustc_attrs)]
mod a{#[rustc_unsafe_specialization_marker]pub trait Fast:Clone{}}mod b{pub trait Fast:Clone{}}impl b::Fast for u8{}use b::Fast;
trait V:Clone{fn f()->u8;}impl<T:Clone>V for T{default fn f()->u8{1}}impl<T:Fast>V for T{fn f()->u8{2}}fn main(){}
