fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait F{type Out<'a>;fn make<'a>(x:&'a u8)->Self::Out<'a>;}struct S;impl F for S{type Out<'a>=&'a u8;fn make<'a>(x:&'a u8)->Self::Out<'a>{x}}fn shorten<'a:'b,'b>(x:<S as F>::Out<'a>)-><S as F>::Out<'b>{x}fn main(){let v=7;check(*shorten(S::make(&v))==7);}
