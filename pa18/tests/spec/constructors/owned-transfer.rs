fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D(u8);struct S<T>(T,T);fn main(){let s=S(D(23),D(24));check(s.0.0==23&&s.1.0==24);}
