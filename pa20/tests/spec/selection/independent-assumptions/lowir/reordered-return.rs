trait A{type Out;}trait B<V>{fn m()->V;}fn f<T:A,U:B<u8>+B<<T as A>::Out>>()->u8{U::m()}fn main(){}
