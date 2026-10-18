fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A where Self::Out:Copy{type Out;}impl A for u8{type Out=&'static mut u8;}fn main(){}