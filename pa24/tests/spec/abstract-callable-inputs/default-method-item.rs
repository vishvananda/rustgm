fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Num{fn num(&self)->u16;}
impl Num for u16{fn num(&self)->u16{*self}}
fn by<T,F:FnMut(&T)->u16>(x:T,mut f:F)->u16{f(&x)}
trait Pick{type Item:Num;fn pick(&self,x:Self::Item)->u16{by(x,Num::num)}}
impl Pick for u8{type Item=u16;}
fn main(){check(7u8.pick(9)==9);}
