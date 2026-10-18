fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;}impl<T:A>A for T{type Out=u8;}fn main(){let _: <u16 as A>::Out=7;}
