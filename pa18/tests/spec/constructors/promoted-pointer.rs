fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);fn main(){let p:&'static fn(u16)->S<u16>=&(S as fn(u16)->S<u16>);let a=(*p)(502);check(a.0==502);}
