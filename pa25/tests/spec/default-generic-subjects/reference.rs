#![feature(const_trait_impl)]
pub const trait Base<Rhs>{fn read(&self,rhs:&Rhs)->u8;}
const impl<A,B>Base<&B> for &A where A:[const]Base<B>{fn read(&self,rhs:&&B)->u8{Base::read(*self,*rhs)}}
pub const trait Probe<X>{fn check<Y>(x:&X,y:&Y)->u8 where X:[const]Base<Y>{Base::read(&x,&y)}}
fn main(){}
