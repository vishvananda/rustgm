fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct A(u8);struct B(u8);impl PartialEq<B> for A{fn eq(&self,b:&B)->bool{self.0==b.0}}fn main(){let a=[A(3)];let b=[B(3)];let s:&[B]=&b;check(a==s);}
