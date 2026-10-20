trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn f<'x,T>(x:&'x u8)-><T as A<'static>>::Out where for<'a>T:A<'a,Out=&'a u8>{x}fn main(){}
