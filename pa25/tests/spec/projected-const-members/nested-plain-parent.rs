#![feature(const_trait_impl)]
pub const trait Base<R>{fn make(value:R)->Self;}
pub const trait Maker<R>{type Output:[const]Base<R>;}
pub const trait Stage<R>{type Next:[const]Maker<R>;}
pub const fn convert<M:Stage<R>,R>(value:R)-><M::Next as Maker<R>>::Output{Base::make(value)}
fn main(){}
