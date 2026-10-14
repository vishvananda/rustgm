fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);fn main(){let f:fn(u16)->S<u16>=S;let a=f(300);check(a.0==300);}
