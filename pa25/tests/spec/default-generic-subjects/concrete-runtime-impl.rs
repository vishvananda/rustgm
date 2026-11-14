#![feature(const_trait_impl)]
pub const trait Base<Rhs>{fn read(&self,rhs:&Rhs)->u8;}
const impl<A,B>Base<&B> for &A where A:[const]Base<B>{fn read(&self,rhs:&&B)->u8{Base::read(*self,*rhs)}}
impl Base<u8> for u8{fn read(&self,rhs:&u8)->u8{*rhs}}
pub const trait Probe{fn check()->u8{Base::read(&7u8,&7u8)}}
fn main(){}
