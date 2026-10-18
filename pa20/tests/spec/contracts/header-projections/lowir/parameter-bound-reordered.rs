fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<T> where T::Out:Copy,T:B{}trait B{type Out;}impl B for u8{type Out=u16;}impl A<u8>for u16{}fn main(){}