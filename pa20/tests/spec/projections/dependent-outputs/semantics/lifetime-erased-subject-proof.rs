fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait B{fn value()->usize;}struct X<const N:usize>;trait A<'a>{type Out;}trait Z{const C:usize;}impl B for X<{<u8 as Z>::C}>{fn value()->usize{3}}impl Z for <&'static u16 as A<'static>>::Out{const C:usize=3;}impl<'a> A<'a> for &'a u16{type Out=u8;}fn main(){check(<X<3>as B>::value()==3);}
