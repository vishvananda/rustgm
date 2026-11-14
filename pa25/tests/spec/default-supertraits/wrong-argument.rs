#![feature(const_trait_impl)]
pub const trait Base<Rhs>{fn read(&self,rhs:&Rhs)->u8;}
pub const trait Derived<Rhs>:[const]Base<u8>{fn through(&self,rhs:&Rhs)->u8{Base::read(self,rhs)}}
fn main(){}
