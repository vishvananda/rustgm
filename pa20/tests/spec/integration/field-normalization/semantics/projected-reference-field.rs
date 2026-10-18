fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;}impl A for u8{type Out=u16;}struct W<'a,T:A>{v:&'a T::Out}fn main(){let x=7u16;let w=W::<u8>{v:&x};check(*w.v==7);}