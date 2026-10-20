trait A{fn f<'a>(x:&'a u8)->impl Copy+'static;}impl A for u8{fn f<'a>(x:&'a u8)->impl Copy+'a{x}}fn main(){}
