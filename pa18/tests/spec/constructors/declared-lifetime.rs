fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a,T>(&'a T);fn wrap<'a,T>(x:&'a T)->S<'a,T>{S(x)}fn main(){let n=18;check(*wrap(&n).0==18);}
