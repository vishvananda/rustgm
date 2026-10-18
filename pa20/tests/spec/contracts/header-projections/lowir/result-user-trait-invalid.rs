fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A where Self::Out:Need{type Out;}trait Need{}impl A for u8{type Out=u16;}fn main(){}