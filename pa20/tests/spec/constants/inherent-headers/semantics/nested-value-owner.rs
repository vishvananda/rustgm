fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}

trait B{fn value()->usize;}struct X<const N:usize>;
impl B for X<{S::C}>{fn value()->usize{3}}struct S;struct U;impl S{const C:usize=U::C;}impl U{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
