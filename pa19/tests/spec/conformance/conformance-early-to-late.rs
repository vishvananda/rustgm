trait A{fn f<'a:'a>(x:&'a u8);}impl A for u8{fn f<'a>(x:&'a u8){}}fn main(){}
