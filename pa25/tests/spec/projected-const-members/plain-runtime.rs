#![feature(const_trait_impl)]
pub const trait Base<R>{fn make(value:R)->Self;}
pub const trait Maker<R>{type Output:[const]Base<R>;}
pub fn convert<M:Maker<R>,R>(value:R)->M::Output{Base::make(value)}
fn main(){}
