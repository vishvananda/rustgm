fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl PartialEq for S{fn eq(&self,r:&Self)->bool{self.0==r.0}}fn main(){check(S(3).eq(&S(3)));check(S(3).ne(&S(4)));check(3u8.eq(&3));check(3u8.ne(&4));}
