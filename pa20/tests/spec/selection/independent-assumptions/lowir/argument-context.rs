trait A{type Out;}trait B<V>{fn m(x:V)->u8;}fn f<T:A,U:B<<T as A>::Out>+B<u8>>(x:u8)->u8{U::m(x)}fn main(){}
