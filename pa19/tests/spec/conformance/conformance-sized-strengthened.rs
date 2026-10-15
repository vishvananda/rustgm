trait A{fn f<T:?Sized>();}impl A for u8{fn f<T>(){}}fn main(){}
