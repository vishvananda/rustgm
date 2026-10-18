fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A:B where Self::Out:Copy{type Out;}trait B:A{}fn main(){}