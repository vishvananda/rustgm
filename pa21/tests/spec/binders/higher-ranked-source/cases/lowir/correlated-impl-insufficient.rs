trait A<'a,'b>{}impl<'a>A<'a,'a>for u8{}fn f<T:for<'a,'b>A<'a,'b>>(){}fn main(){f::<u8>();}
