fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);impl<T>S<T>{const C:u8=7;}fn f(x:u8)->bool{match x{S::<u16>::C=>true,_=>false}}fn main(){check(f(7));check(!f(9))}