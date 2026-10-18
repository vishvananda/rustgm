fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait B{type Out;}trait A:B{type Out;fn keep(x:Self::Out)->Self::Out{x}}fn main(){}