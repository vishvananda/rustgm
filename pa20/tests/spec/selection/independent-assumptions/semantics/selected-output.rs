trait A{type Out:Copy;}trait B<V>{fn m()->V;}fn f<T:A,U:B<<T as A>::Out>+B<u8>>()-><T as A>::Out{U::m()}fn main(){}
