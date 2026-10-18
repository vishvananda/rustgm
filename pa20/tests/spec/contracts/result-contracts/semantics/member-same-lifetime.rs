fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<'a>{type Out:'a;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn main(){let x=7u8;let y:<u8 as A<'_>>::Out=&x;check(*y==7);}