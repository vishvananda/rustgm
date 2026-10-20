trait A{fn f<T>(x:T)->impl Copy;}impl A for u8{fn f<T:Copy>(x:T)->impl Copy{x}}fn main(){}
