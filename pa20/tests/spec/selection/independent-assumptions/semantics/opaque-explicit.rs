trait A{type Out;}trait B<U>{}trait C{type Out;}impl<T:B<u8>>C for T{type Out=u16;}struct S<T:A,U:B<<T as A>::Out>+B<u8>>where<U as C>::Out:Copy{t:T,u:U}fn main(){}
