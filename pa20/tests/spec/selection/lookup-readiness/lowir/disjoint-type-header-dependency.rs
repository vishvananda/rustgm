fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}

trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{S::<u8,3>::C}>{fn value()->usize{3}}struct S<T,const N:usize>(T);impl S<u16,{S::<u8,3>::C}>{const C:usize=9;}impl S<u8,3>{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
