fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;}impl A for u8{type Out=u16;}#[repr(transparent)]struct W<T:A>{v:T::Out,w:T::Out}fn main(){}