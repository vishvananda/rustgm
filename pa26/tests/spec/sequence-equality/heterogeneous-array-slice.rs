fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct A(u8);struct B(u8);impl PartialEq<B> for A{fn eq(&self,b:&B)->bool{self.0==b.0}}fn eq(a:[A;1],b:&[B])->bool{a==*b}fn main(){check(eq([A(3)],&[B(3)]));}
