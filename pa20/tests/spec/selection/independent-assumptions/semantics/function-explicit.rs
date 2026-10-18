trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<U>{}trait C{type Out;}impl<T:B<u8>>C for T{type Out=u16;}fn f<T:Copy,U:B<<T as A>::Out>+B<u8>>()where<U as C>::Out:Copy{}fn main(){}
