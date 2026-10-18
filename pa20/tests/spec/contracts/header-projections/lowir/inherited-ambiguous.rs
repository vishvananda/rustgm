fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A:B+C where Self::Out:Copy{}trait B{type Out;}trait C{type Out;}fn main(){}