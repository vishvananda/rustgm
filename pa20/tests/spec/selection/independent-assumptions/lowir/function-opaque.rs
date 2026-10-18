trait A{type Out;}trait B<U>{}fn need<U:B<u8>>(){}fn f<T:A,U:B<<T as A>::Out>+B<u8>>(){need::<U>();}fn main(){}
