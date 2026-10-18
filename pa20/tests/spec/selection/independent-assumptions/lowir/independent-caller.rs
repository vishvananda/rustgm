trait A{type Out;}trait B<V>{fn m()->V;}fn f<T:A,U:B<<T as A>::Out>+B<u8>>()->u8{U::m()}fn g<T:A,U:B<<T as A>::Out>+B<u16>>()->u8{U::m()}fn main(){}
