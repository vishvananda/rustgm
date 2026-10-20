fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait F{type Out<'a>;}struct S;impl F for S{type Out<'a>=&'a u8;}fn get<'a,T:for<'b>F<Out<'b>=&'b u8>>(x:T::Out<'a>)->u8{*x}fn main(){let v=7;check(get::<S>(&v)==7);}
