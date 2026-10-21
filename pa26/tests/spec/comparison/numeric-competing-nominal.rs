fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl PartialEq<S> for u8{fn eq(&self,r:&S)->bool{*self==r.0}}fn main(){check(7u8.eq(&7));check(7u8.eq(&S(7)));check(7u8.ne(&8));check(7u8.ne(&S(8)));}
