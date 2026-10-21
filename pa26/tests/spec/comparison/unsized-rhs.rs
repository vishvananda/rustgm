fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S;impl PartialEq<str> for S{fn eq(&self,_:&str)->bool{true}}fn main(){let s="hello";check(S==*s);check(!<S as PartialEq<str>>::ne(&S,s));}
