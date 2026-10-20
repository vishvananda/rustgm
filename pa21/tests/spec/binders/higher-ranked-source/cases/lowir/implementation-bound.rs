trait A<'a>{}trait B{}impl<'a>A<'a>for u8{}impl<T>B for T where for<'a>T:A<'a>{}fn f<T:B>(){}fn main(){f::<u8>();}
