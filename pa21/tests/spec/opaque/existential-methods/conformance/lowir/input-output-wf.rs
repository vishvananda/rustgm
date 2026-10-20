trait A{fn f<'a,T>(x:&'a T)->impl Sized+'a;}impl A for u8{fn f<'a,T>(x:&'a T)->&'a T{x}}fn main(){}
