fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);fn wrap<T>(x:T)->S<T>{S(x)}fn main(){let a=wrap(504u16);check(a.0==504);}
