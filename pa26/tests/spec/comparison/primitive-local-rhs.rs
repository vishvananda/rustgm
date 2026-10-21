fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl PartialEq<S> for u8{fn eq(&self,r:&S)->bool{*self==r.0}}fn main(){check(7==S(7));check(7!=S(8));}
