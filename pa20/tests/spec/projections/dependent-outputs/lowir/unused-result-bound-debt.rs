fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait B{fn value()->usize;}struct X<const N:usize>;trait A{type Out:Copy;}struct Bad;impl B for X<{<u8 as Z>::C}>{fn value()->usize{3}}trait Z{const C:usize;}impl Z for <u16 as A>::Out{const C:usize=3;}impl A for u16{type Out=u8;}impl A for u32{type Out=Bad;}fn main(){check(<X<3>as B>::value()==3);}
