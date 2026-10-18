fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A:B<<Self as A>::Out>{type Out;}trait B<T>{}impl B<u16>for u8{}impl A for u8{type Out=u16;}fn main(){}