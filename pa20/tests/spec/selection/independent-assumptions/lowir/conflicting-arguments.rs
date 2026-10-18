trait A{type Out;}trait B<V,W>{fn m(x:V)->W;}fn f<T:A,U:B<<T as A>::Out,<T as A>::Out>+B<u8,u8>>()->u16{U::m(7u8)}fn main(){}
