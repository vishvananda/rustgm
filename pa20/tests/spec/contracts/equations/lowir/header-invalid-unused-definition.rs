fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A where Self::Out:B<Out=u8>{type Out;}trait B{type Out;}impl A for u8{type Out=u16;}impl B for u16{type Out=u32;}fn main(){}