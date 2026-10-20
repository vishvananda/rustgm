trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=u16;}fn f<T>()where for<'a>T:A<'a,Out=&'a u8>{}fn main(){f::<u8>();}
