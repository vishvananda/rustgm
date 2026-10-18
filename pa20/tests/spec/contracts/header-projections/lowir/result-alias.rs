fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type Out<T:?Sized+A>=<T as A>::Out;trait A where Out<Self>:Copy{type Out;}impl A for u8{type Out=u16;}fn main(){}