fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A:B<Self::Out>{type Out;}trait B<T>:A{}fn main(){}