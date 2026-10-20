trait A{}impl A for u16{}struct W<T:?Sized>(u8,T);const P:&W<dyn A>=&W(2,7u8);fn main(){}
