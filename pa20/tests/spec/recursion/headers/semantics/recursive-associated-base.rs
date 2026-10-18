fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;const C:u8;}impl A for u8{type Out=u8;const C:u8=7;}struct W<T>(T);impl<T:A>A for W<T>{type Out=T::Out;const C:u8=T::C;}fn main(){let x:<W<W<u8>>as A>::Out=7;check(x==7&&<W<W<u8>>as A>::C==7)}
