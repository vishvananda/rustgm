trait A{fn f<'a>(x:&'a u8)->impl Copy+use<Self>;}impl A for u8{fn f<'a>(x:&'a u8)->impl Copy{x}}fn main(){}
