trait A<'a>{type Out;const C:<Self as A<'a>>::Out;}impl<'a>A<'a>for u8{type Out=&'a u8;const C:&'a u8=&7;}fn main(){}
