fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl core::cmp::PartialEq<&S>for&S{fn eq(&self,r:&&S)->bool{self.0==r.0}}fn main(){let a=S(3);let b=S(3);check(<&S as core::cmp::PartialEq<&S>>::eq(&&a,&&b));}
