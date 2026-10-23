fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl S{fn make(x:u8)->Self{Self(x)}}fn main(){check(S::make(7).0==7);}
