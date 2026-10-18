fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;}impl<T>A for T{type Out=u8;}impl A for u16{type Out=u32;}fn main(){}
