fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A where <Self as B>::Out:Copy{}trait B{type Out;}fn main(){}