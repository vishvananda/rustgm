trait A<T,U>{const C:(T,U);}impl A<u8,u16>for u32{const C:(u8,u16)=(7,9);}fn main(){}
