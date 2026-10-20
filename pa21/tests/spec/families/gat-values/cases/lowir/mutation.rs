fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait F{type Out<'a>;fn make<'a>(x:&'a mut u8)->Self::Out<'a>;}struct S;impl F for S{type Out<'a>=&'a mut u8;fn make<'a>(x:&'a mut u8)->Self::Out<'a>{x}}fn main(){let mut v=7;*S::make(&mut v)=11;check(v==11);}
