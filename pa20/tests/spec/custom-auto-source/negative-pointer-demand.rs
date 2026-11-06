#![feature(auto_traits,negative_impls)]
auto trait Auto{}trait Allowed{}impl<T:Allowed> !Auto for *const T{}
fn need<T:Auto>(){} pub fn check(){need::<*const u8>();}
