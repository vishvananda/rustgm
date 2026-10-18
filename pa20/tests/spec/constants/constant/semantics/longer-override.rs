trait A<'a>{const C:&'a u8;}impl<'a>A<'a>for u8{const C:&'static u8=&7;}fn main(){}
