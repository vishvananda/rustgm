#![feature(const_trait_impl)]
pub const trait Base<Rhs>{fn read(&self,rhs:&Rhs)->u8;}
pub const trait Probe<X>{fn check0<Y>(x:&X,y:&Y)->u8 where X:[const]Base<Y>{Base::read(x,y)}fn check1<Y>(x:&X,y:&Y)->u8 where X:[const]Base<Y>{Base::read(x,y)}fn check2<Y>(x:&X,y:&Y)->u8 where X:[const]Base<Y>{Base::read(x,y)}fn check3<Y>(x:&X,y:&Y)->u8 where X:[const]Base<Y>{Base::read(x,y)}fn check4<Y>(x:&X,y:&Y)->u8 where X:[const]Base<Y>{Base::read(x,y)}fn check5<Y>(x:&X,y:&Y)->u8 where X:[const]Base<Y>{Base::read(x,y)}fn check6<Y>(x:&X,y:&Y)->u8 where X:[const]Base<Y>{Base::read(x,y)}fn check7<Y>(x:&X,y:&Y)->u8 where X:[const]Base<Y>{Base::read(x,y)}}
fn main(){}
