trait A<'a>{type Out;}impl<'a,'x>A<'a>for &'x u8{type Out=&'x u8;}fn f<T>()where for<'a>T:A<'a>,for<'a><T as A<'a>>::Out:'a{}fn main(){f::<&'static u8>();}
