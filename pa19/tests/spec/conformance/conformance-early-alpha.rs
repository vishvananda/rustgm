trait A{fn f<'a:'a>(x:&'a u8);}impl A for u8{fn f<'z:'z>(x:&'z u8){}}fn main(){}
