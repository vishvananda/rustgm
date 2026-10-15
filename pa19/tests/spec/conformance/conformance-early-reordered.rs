trait A{fn f<'a:'a,'b:'b>(x:&'a u8,y:&'b u8)->&'a u8;}impl A for u8{fn f<'b:'b,'a:'a>(x:&'a u8,y:&'b u8)->&'a u8{x}}fn main(){}
