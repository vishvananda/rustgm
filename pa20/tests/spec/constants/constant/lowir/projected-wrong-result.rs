trait A{type Out;const C:<Self as A>::Out;}impl A for u8{type Out=u16;const C:u8=7;}fn main(){}
