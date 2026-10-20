trait Read<'a>{}struct S<'a>(&'a u8);impl Read<'static> for S<'static>{}fn f<'b>(x:&'b u8)->impl for<'a>Read<'a>{S(x)}fn main(){}
