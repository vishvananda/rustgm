fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;fn f(x:Self::Out)where Self:B{}}trait B{type Out;}fn main(){}