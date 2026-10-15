trait A{fn f<T>();}impl A for u8{fn f<T:?Sized>(){}}fn main(){}
