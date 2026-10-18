fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;const C:u8;}struct W<T>(T);impl<T:A>A for W<T>{type Out=T::Out;const C:u8=T::C;}fn main(){let _: <W<W<u8>>as A>::Out=7;}
