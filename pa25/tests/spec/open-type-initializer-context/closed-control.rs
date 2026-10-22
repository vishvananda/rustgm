fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);impl<T>S<T>{const C:u8={let n=7;n};}fn main(){check(S::<&'static u8>::C==7);}
