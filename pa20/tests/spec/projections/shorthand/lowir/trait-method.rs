fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;fn keep(x:Self::Out)->Self::Out{x}}impl A for u8{type Out=u16;}fn main(){check(<u8 as A>::keep(7)==7);}