trait A<'a>{}impl<'a>A<'a>for &'a u8{}fn f<T>()where for<'a>&'a T:A<'a>{}fn main(){f::<u8>();}
