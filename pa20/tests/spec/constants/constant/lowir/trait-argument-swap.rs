trait A<T,U>{const C:(T,U);}impl A<u8,u16>for u32{const C:(u16,u8)=(7,9);}fn main(){}
