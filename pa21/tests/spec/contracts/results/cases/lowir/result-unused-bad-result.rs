trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'static u8;}trait Family{type Item:for<'a>A<'a,Out=&'a u8>;}impl Family for u16{type Item=u8;}fn main(){}
