fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl PartialEq for S{fn eq(&self,r:&Self)->bool{self.0==r.0}}fn main(){let mut a=S(1);let _=a=={a.0=2;S(2)};}
