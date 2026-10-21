fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);impl PartialEq<u8> for S{fn eq(&self,r:&u8)->bool{self.0==*r}}fn eq<L:PartialEq<R>,R>(l:L,r:R)->bool{l==r}fn main(){check(eq(S(4),4u8));}
