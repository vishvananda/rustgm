fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl PartialEq<u8> for S{fn eq(&self,r:&u8)->bool{self.0==*r}}fn main(){let s=S(7);check(s==7u8);check(s!=8u8);check(s.0==7);}
