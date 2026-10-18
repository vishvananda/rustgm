fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait I{type Out;}impl I for u8{type Out=u16;}trait A{type Out;}impl A for <u8 as I>::Out{type Out=u8;}impl A for u16{type Out=u32;}fn main(){}
