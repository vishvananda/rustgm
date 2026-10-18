trait A{type Out;}trait B<V>{fn m(x:V)->u8;}trait C{fn m(x:u16)->u8;}fn f<T:A,U:B<<T as A>::Out>+B<u8>+C>()->u8{U::m(7u8)}fn main(){}
