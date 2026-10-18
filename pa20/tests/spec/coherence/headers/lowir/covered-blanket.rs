fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;}struct W<T>(T);impl<T>A for W<T>{type Out=T;}fn main(){let x:<W<u16>as A>::Out=7;check(x==7)}
