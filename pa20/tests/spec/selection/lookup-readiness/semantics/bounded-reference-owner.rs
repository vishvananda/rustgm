fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}

trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{S::<&'static u8>::C}>{fn value()->usize{3}}struct S<T>(T);trait P<'a>{}impl<'a,T:P<'a>>S<&'a T>{const C:usize=3;}trait Unused{const C:usize=9;}impl<'a>P<'a>for u8{}fn main(){check(<X<3>as B>::value()==3);}
