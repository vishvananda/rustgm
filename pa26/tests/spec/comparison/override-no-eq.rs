fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct A(u8);struct B;impl PartialEq for A{fn eq(&self,r:&Self)->bool{self.0==r.0}}impl PartialEq for B{fn eq(&self,_:&Self)->bool{let _=fail(0);true}fn ne(&self,_:&Self)->bool{false}}fn main(){check(A(1)!=A(2));check(!(A(1)!=A(1)));check(!(B!=B));}
