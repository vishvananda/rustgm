trait A{fn f<'a>(x:&'a u8)->impl Copy+'a;}impl A for u8{fn f<'b>(x:&'b u8)->impl Copy+'b{x}}fn main(){}
