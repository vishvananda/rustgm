trait A<'a>:'a{}impl<'a,'x:'a>A<'a>for &'x u8{}trait Family<'x>{type Item:for<'a>A<'a>;}impl<'x>Family<'x>for &'x u8{type Item=&'x u8;}fn main(){}
