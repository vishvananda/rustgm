fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl PartialEq for S{fn eq(&self,r:&Self)->bool{self.0==r.0}}impl Eq for S{}fn equal<T:Eq>(a:T,b:T)->bool{a==b}fn main(){check(equal(S(5),S(5)));check(equal(9i16,9i16));}
