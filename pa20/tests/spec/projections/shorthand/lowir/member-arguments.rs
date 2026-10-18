fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;fn keep(x:Self::Out<u8>);}fn main(){}