fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl PartialEq for S{fn eq(&self,r:&Self)->bool{self.0==r.0}}fn main(){let a=S(4);let b=S(4);let c=||a==b;check(c());check(c());check(a.0==4&&b.0==4);}
