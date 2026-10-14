fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);fn wrap<'a>(x:&'a u8)->S<&'a u8>{S(x)}fn main(){let n=17;check(*wrap(&n).0==17);}
