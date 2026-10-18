fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=u8;}impl A<'static>for u8{type Out=u16;}fn main(){}
