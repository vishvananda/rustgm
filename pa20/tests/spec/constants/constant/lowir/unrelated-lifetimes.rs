trait A<'a>{const C:&'a u8;}impl<'a,'b>A<'a>for &'b u8{const C:&'b u8=&7;}fn main(){}
