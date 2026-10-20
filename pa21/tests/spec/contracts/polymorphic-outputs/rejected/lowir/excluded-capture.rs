trait Read<'a>{}struct S<'b>(&'b u8);impl<'a,'b>Read<'a> for S<'b>{}fn f<'b>(x:&'b u8)->impl for<'a>Read<'a>+use<>{S(x)}fn main(){}
