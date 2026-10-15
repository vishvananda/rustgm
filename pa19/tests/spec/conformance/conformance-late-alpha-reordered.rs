trait A{fn f<'a,'b>(x:&'a u8,y:&'b u8)->&'a u8;}impl A for u8{fn f<'b,'a>(x:&'a u8,y:&'b u8)->&'a u8{x}}fn main(){}
