trait A<'a>{}trait B:for<'a>A<'a>{}impl<'a>A<'a>for u8{}impl B for u8{}fn need<'x,T:A<'x>>(){}fn f<T:B>(){need::<'static,T>();}fn main(){f::<u8>();}
