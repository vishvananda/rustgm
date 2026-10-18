trait B<'a,U>{}fn need<'a,U:B<'a,u8>>(_: &'a u8){}fn f<'a,'b,U:B<'a,u8>+B<'b,u8>>(x:&'a u8,y:&'b u8){need::<U>(x);}fn main(){}
