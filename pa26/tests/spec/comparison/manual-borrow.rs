fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl PartialEq for S{fn eq(&self,r:&Self)->bool{self.0==r.0}}fn main(){let a=S(7);let b=S(7);check(a==b);check(a.0==7&&b.0==7);}
