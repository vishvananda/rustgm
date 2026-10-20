trait A<'a,'b>{}impl<'a,'b>A<'a,'b>for u8{}fn f<T:for<'a,'b>A<'a,'b>>(){}fn main(){f::<u8>();}
