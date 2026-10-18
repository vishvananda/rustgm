trait A{type Out;}trait B<'a,V>{fn m(x:&'a u8)->V;}fn f<'a,'b:'a,T:A,U:B<'a,u8>+B<'b,<T as A>::Out>>(x:&'b u8)->u8{U::m(x)}fn main(){}
