fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait B{fn value()->usize;}struct X<const N:usize>;trait A{type Out;const C:Self::Out;}impl B for X<{<u8 as A>::C}>{fn value()->usize{3}}impl A for u8{type Out=usize;const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
