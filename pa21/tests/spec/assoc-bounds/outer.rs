trait A<'a>{type Out;}trait B<'a>{}struct S;impl<'a>A<'a> for S{type Out=&'a u8;}impl<'a>B<'a> for &'a u8{}fn f<T:for<'a>A<'a,Out:B<'a>>>(){}fn main(){f::<S>();}
