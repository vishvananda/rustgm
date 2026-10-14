fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);fn main(){let f=S;let a=f(9u8);check(a.0==9);}
