fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait F{type Out<U>;}trait A:F{}struct S;impl F for S{type Out<U>=U;}impl A for S{}fn get<T:A<Out<u8>=u8>>(x:T::Out<u8>)->u8{x}fn main(){check(get::<S>(7)==7);}
