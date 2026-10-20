fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait F{type Out<T>;fn make<T>(x:T)->Self::Out<T>;}struct S;impl F for S{type Out<T>=T;fn make<T>(x:T)->Self::Out<T>{x}}fn main(){check(S::make(7u8)==7);check(S::make(11u16)==11);}
