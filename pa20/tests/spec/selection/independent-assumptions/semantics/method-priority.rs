trait A{type Out;}trait B<V>{fn m()->V;}trait C{fn m()->u8;}fn f<T:A,U:B<<T as A>::Out>+B<u8>+C>()->u8{U::m()}fn main(){}
