trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn bad<'a>(x:&'a u8)-><u8 as A<'static>>::Out{x}fn main(){}
