fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;}struct X;struct Y;impl A for X{type Out=u8;}impl A for Y{type Out=u16;}fn main(){let x:<X as A>::Out=7;let y:<Y as A>::Out=9;check(x==7&&y==9)}
