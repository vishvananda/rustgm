trait A{type Out;}impl<T:Copy>A for T{type Out=u8;}trait B<'a,U>{}fn need<'a,U:B<'a,u8>>(_: &'a u8){}fn f<'a,'b,T:Copy,U:B<'a,u8>+B<'b,<T as A>::Out>>(x:&'a u8,y:&'b u8){need::<U>(x);}fn main(){}
