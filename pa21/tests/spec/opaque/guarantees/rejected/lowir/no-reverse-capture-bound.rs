fn f<'a>(_:&'a u8)->impl Copy+'static{7u8}fn g<'a>(x:&'a u8)->&'static u8{let _=f(x);x}fn main(){}
