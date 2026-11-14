#![feature(const_trait_impl)]
pub const trait Base<R>{fn make(value:R)->Self;}
pub const trait Maker{type Output:[const]Base<u16>;}
pub const fn convert<M:[const]Maker>(value:u8)->M::Output{Base::make(value)}
fn main(){}
