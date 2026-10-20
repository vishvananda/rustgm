trait A{}impl A for &u8{}struct W<T:?Sized>(u8,T);fn bad<'a>(x:&'a u8){let p:&W<dyn A+'static>=&W(2,x);}fn main(){}
