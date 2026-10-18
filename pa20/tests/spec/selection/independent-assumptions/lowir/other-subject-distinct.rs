trait A{type Out;}trait B<V>{fn m()->V;}fn f<T:A,U:B<<T as A>::Out>+B<u8>>()->u8{T::m()}fn main(){}
