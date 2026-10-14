fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);fn main(){let a=S(7u8);let b=S(300u16);check(a.0==7&&b.0==300);}
