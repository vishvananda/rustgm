#![feature(auto_traits,negative_impls)]
auto trait Auto{}
impl<'a0> !Auto for &'a0 mut u8{}
fn need<T:Auto>(){}
pub fn reject(){need::<&mut u8>();}
