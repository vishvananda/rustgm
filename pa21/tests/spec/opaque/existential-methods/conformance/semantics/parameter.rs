trait A{fn f<T:Copy>(x:T)->impl Copy;}impl A for u8{fn f<U:Copy>(x:U)->impl Copy{x}}fn main(){}
