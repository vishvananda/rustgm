#![feature(const_trait_impl)]
pub const trait Base<R>{fn make(value:R)->Self;}
pub const trait Maker{type Output:for<'a>[const]Base<&'a u8>;}
pub const fn convert<'a,M:[const]Maker>(value:&'a u8)->M::Output{Base::make(value)}
fn main(){}
