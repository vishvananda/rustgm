#![feature(const_trait_impl)]
pub const trait Base<Rhs>{fn read(&self,rhs:&Rhs)->u8;}
pub const trait Probe<X>{fn read(&self)->u8;fn check<Y>(y:&Y)->u8 where Y:[const]Probe<X>{Probe::read(y)}}
fn main(){}
