#![feature(const_trait_impl)]
pub const trait Base<Rhs>{fn read(&self,rhs:&Rhs)->u8;}
pub const trait Probe<X:[const]Base<u8>>{fn check(x:&X,y:&u8)->u8{Base::read(x,y)}}
fn main(){}
