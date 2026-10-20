trait A<'a>{fn f()->impl Copy+use<'a,Self>;}impl<'a>A<'a> for u8{fn f()->impl Copy{7u8}}fn main(){}
