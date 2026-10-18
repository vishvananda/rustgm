trait A{type Out;}trait B<'a,U>{}fn need<'a,U:B<'a,u8>>(_: &'a u8){}fn f<'a,'b,T:A,U:B<'a,u8>+B<'b,<T as A>::Out>>(x:&'a u8,y:&'b u8)where 'a:'b,'b:'a{need::<'b,U>(y);}fn main(){}
