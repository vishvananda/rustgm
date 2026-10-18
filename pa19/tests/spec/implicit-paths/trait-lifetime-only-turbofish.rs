fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a>(&'a u8);trait A<'a>{fn value()->usize;}impl<'a>A<'a>for S<'a>{fn value()->usize{3}}fn read<'a>(_:&'a u8)->usize{S::<'a>::value()}fn main(){let x=1u8;check(read(&x)==3);}
