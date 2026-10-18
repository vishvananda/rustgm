fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}

trait B{fn value()->usize;}struct X<const N:usize>;
impl B for X<{S::<u8>::C}>{fn value()->usize{3}}struct S<T>(T);impl S<u16>{const C:usize=9;}impl S<u8>{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
