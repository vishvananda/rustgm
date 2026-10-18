fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a>(&'a u8);impl<'a>S<'a>{fn value()->usize{3}}fn main(){let _=S::<'static,'static>::value();}
