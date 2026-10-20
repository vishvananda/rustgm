trait A<'a>{}trait B<'a>{}impl<'a>A<'a>for &'a u8{}impl<'a>B<'a>for &'a u8{}fn f<T>()where for<'a>&'a T:A<'a>+B<'a>{}fn main(){f::<u8>();}
