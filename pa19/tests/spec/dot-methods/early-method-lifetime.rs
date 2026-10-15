fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S;impl S{fn f<'a:'a>(&self,x:&'a u8)->&'a u8{x}}fn main(){let n=7u8;check(*S.f::<'_>(&n)==7);}