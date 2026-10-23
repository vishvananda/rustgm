fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a>(&'a u8);impl<'a>S<'a>{fn make(x:&'a u8)->Self{Self(x)}}fn main(){let n=7;check(*S::make(&n).0==7);}
