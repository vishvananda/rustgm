trait A{fn f<const N:u8>();}impl A for u8{fn f<const N:u16>(){}}fn main(){}
