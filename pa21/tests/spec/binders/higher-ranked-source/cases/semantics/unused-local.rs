trait A<'a>{}impl<'a>A<'a>for u8{}fn f<T:for<'unused,'a>A<'a>>(){}fn main(){f::<u8>();}
