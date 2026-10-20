fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait F{type Out<U>;}struct S;impl F for S{type Out<U>=U;}struct Wrap<T:F>{v:T::Out<u8>}fn main(){let x=Wrap::<S>{v:7};check(x.v==7);}
