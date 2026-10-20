trait A<X>{}fn f<'x,T>(x:&'x T)where for<'a>T:A<for<'b>fn(&'a u8,&'b u8,&'x u8)>{}fn main(){}
