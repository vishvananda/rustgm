trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'static u8;}fn f<T>()where for<'a>T:A<'a,Out=&'a u8>{}fn main(){f::<u8>();}
