fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out:A;}impl A for u8{type Out=u8;}fn main(){}