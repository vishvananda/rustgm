trait A{fn f<'a,T:'a>();}impl A for u8{fn f<'a:'a,T>(){}}fn main(){}
