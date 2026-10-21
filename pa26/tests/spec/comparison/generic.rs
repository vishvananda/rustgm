fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl PartialEq for S{fn eq(&self,r:&Self)->bool{self.0==r.0}}fn equal<T:PartialEq>(a:T,b:T)->bool{let v=a==b;let _=&a;let _=&b;v}fn main(){check(equal(S(7),S(7)));check(equal(8u16,8u16));}
