trait A{type Out;}trait B<U>{fn m()->U;}fn f<T:A,U:B<<T as A>::Out>+B<u8>>()->u8{U::m()}fn main(){}
