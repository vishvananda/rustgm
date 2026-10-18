fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait P{}trait Q{}trait A{type Out;}impl<T:P>A for T{type Out=u8;}impl<T:Q>A for T{type Out=u16;}fn main(){}
