fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait F{type Out<U>;}struct S;impl F for S{type Out<U>=U;}fn get<T:F<Out<u8>=u8>>(x:T::Out<u8>)->u8{x}fn main(){check(get::<S>(7)==7);}
