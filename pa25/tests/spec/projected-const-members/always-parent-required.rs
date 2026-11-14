#![feature(const_trait_impl)]
pub const trait Base<R>{fn make(value:R)->Self;}
pub const trait Maker<R>{type Output:[const]Base<R>;}
pub const fn require<T:const Base<u8>>(){}
pub const fn convert<M:const Maker<u8>>(){require::<M::Output>()}
fn main(){}
