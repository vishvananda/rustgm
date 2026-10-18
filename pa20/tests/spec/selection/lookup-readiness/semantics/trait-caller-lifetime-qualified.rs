fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}

trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{<S<'static>>::C}>{fn value()->usize{3}}struct S<'a>(&'a u8);trait A<'a>{const C:usize;}impl<'a>A<'a>for S<'a>{const C:usize=3;}fn read<'a>(_:&'a u8)->usize{<S<'a>>::C}fn main(){check(<X<3>as B>::value()==3);let x=1u8;check(read(&x)==3);}
