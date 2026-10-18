fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;}impl<'a>A for &'a u8{type Out=u8;}struct W<T:A>{v:T::Out}fn widen<'a>(w:W<&'a u8>)->W<&'static u8>{w}fn main(){}