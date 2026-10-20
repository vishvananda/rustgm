trait A{fn f()->impl Copy;}impl A for u8{fn f()->u8{7}}fn bad<T:A>(){let _:u8=T::f();}fn main(){}
