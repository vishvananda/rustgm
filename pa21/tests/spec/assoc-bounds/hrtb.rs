trait A{type Out;}trait B<'a>{}struct S;impl A for S{type Out=u8;}impl<'a>B<'a> for u8{}fn want<T:for<'a>B<'a>>(){}fn f<T:A<Out:for<'a>B<'a>>>(){want::<T::Out>();}fn main(){f::<S>();}
