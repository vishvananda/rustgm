fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A where <u8 as B>::Out:Copy{}trait B{type Out;}impl B for u8{type Out=&'static mut u8;}fn main(){}