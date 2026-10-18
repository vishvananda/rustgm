trait A<'a>{const C:&'a mut &'a u8;}impl<'a>A<'a>for u8{const C:&'static mut &'static u8=&mut &7;}fn main(){}
