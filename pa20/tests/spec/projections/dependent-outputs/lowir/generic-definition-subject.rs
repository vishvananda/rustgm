fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait B{fn value()->usize;}struct X<const N:usize>;trait A{type Out;}trait Z{const C:usize;}struct S<T>(T);impl B for X<{<S<u8>as Z>::C}>{fn value()->usize{3}}impl Z for <u8 as A>::Out{const C:usize=3;}impl A for u8{type Out=S<u8>;}fn main(){check(<X<3>as B>::value()==3);}
