fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);static C:fn(u16)->S<u16>=S;fn main(){let a=C(501);check(a.0==501);}
