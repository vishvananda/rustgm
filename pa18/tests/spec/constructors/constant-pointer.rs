fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);const C:fn(u16)->S<u16>=S;fn main(){let a=C(500);check(a.0==500);}
