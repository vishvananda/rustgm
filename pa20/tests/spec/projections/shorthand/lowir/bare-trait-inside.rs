fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;fn keep(x:A::Out)->A::Out{x}}fn main(){}